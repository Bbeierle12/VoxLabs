//! Tap consumers: what reads the wires after the stages, on the worker's
//! observer hook or from the shell's taps, without being a stage. The
//! `VocalProfile` the UI and the harness consume is assembled here from
//! the pipeline's wires — the one place the per-hop outputs become the
//! pre-pipeline profile shape — so `FrameAnalyzer`'s consumers keep their
//! contract while the analyzer itself retires.

use crate::types::{VocalProfile, VoiceMetrics};

use super::types::{F0Track, FormantTrack, HarmonicSeries, Wire};

/// The latest wire of type `T` (the last producer's output).
pub fn latest<T: super::types::WireValue>(wires: &[Wire]) -> Option<&T> {
    wires.iter().rev().find_map(T::from_wire)
}

/// The first wire of type `T` (the first producer's output — for `F0Track`,
/// the estimator's verdict before the voicing gates).
pub fn first<T: super::types::WireValue>(wires: &[Wire]) -> Option<&T> {
    wires.iter().find_map(T::from_wire)
}

/// Assembles the profile from the gated f0, the held formants, the
/// harmonic series and the voice metrics — the exact fields
/// `FrameAnalyzer::analyze` fills. `None` when a required wire is absent
/// (the mode file did not include that stage).
pub fn profile_from_wires(wires: &[Wire]) -> Option<VocalProfile> {
    let f0: &F0Track = latest(wires)?;
    let formants: &FormantTrack = latest(wires)?;
    let harmonics: &HarmonicSeries = latest(wires)?;
    let metrics: &VoiceMetrics = latest(wires)?;
    Some(VocalProfile {
        f0: f0.hz,
        formants: formants.formants,
        formants_f0: formants.measured_f0,
        partial_amplitudes: harmonics.amplitudes,
        metrics: *metrics,
        valid: f0.voiced,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{ANALYSIS_FRAME, FrameAnalyzer};
    use crate::pipeline::builder::build;
    use crate::pipeline::definition::PipelineDefinition;
    use crate::pipeline::types::AudioFrame;
    use std::f32::consts::TAU;

    const SR: f32 = 48_000.0;

    /// The Live Model at hop = frame (one hop per analysis frame), so the
    /// contour tracker sees the analyzer's cadence, with the coarse grid.
    fn frame_cadence_live_model() -> PipelineDefinition {
        let text = format!(
            "{}\n[params.tract]\ngrid_n = 21\n",
            PipelineDefinition::LIVE_MODEL
        )
        .replace("hop = 1024", &format!("hop = {ANALYSIS_FRAME}"));
        PipelineDefinition::from_toml(&text).expect("live_model at frame cadence")
    }

    /// The Phase 2 contract: the whole chain reproduces `FrameAnalyzer`'s
    /// profile field for field — f0, formants, harmonics, every metric —
    /// on a voiced/unvoiced sequence, so the analyzer can retire.
    #[test]
    fn pipeline_profile_equals_frame_analyzer_profile() {
        crate::room::reset();
        let def = frame_cadence_live_model();
        let format = def.format(Some(SR));
        let mut pipeline = build(&def, format).expect("build");
        let mut analyzer = FrameAnalyzer::new(SR);

        // Quiet noise (teaches the floor), a harmonic vowel gliding
        // slightly (vibrato-like), then silence.
        let mut seed = 0x2545_F491_4F6C_DD1Du64;
        let mut sig: Vec<f32> = (0..ANALYSIS_FRAME * 40)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                0.002 * ((seed >> 40) as f32 / (1u64 << 24) as f32 - 0.5)
            })
            .collect();
        sig.extend((0..ANALYSIS_FRAME * 60).map(|i| {
            let t = i as f32 / SR;
            let f0 = 150.0 * (1.0 + 0.01 * (TAU * 5.5 * t).sin());
            (1i32..=10)
                .map(|k| (0.6f32).powi(k - 1) * (TAU * f0 * k as f32 * t).sin())
                .sum::<f32>()
                * 0.2
        }));
        sig.extend(std::iter::repeat_n(0.0, ANALYSIS_FRAME * 3));

        let mut voiced_frames = 0;
        for (i, chunk) in sig.chunks_exact(ANALYSIS_FRAME).enumerate() {
            let expected = analyzer.analyze(chunk).profile;
            if let Wire::AudioFrame(src) = &mut pipeline.wires[0] {
                src.samples.copy_from_slice(chunk);
                src.frame_index = i as u64;
            }
            pipeline
                .run_hop()
                .unwrap_or_else(|(s, e)| panic!("stage {s}: {e}"));
            let got = profile_from_wires(&pipeline.wires).expect("profile wires");
            assert_eq!(got.valid, expected.valid, "frame {i} valid");
            assert_eq!(got.f0, expected.f0, "frame {i} f0");
            assert_eq!(got.formants, expected.formants, "frame {i} formants");
            assert_eq!(
                got.formants_f0, expected.formants_f0,
                "frame {i} formants_f0"
            );
            assert_eq!(
                got.partial_amplitudes, expected.partial_amplitudes,
                "frame {i} harmonics"
            );
            let (m, e) = (got.metrics, expected.metrics);
            assert_eq!(m.hnr_db, e.hnr_db, "frame {i} hnr");
            assert_eq!(m.h1_h2_db, e.h1_h2_db, "frame {i} h1h2");
            assert_eq!(m.jitter_pct, e.jitter_pct, "frame {i} jitter");
            assert_eq!(m.shimmer_db, e.shimmer_db, "frame {i} shimmer");
            assert_eq!(m.cpp_db, e.cpp_db, "frame {i} cpp");
            assert_eq!(m.centroid_hz, e.centroid_hz, "frame {i} centroid");
            assert_eq!(m.snr_db, e.snr_db, "frame {i} snr");
            assert_eq!(m.voiced_but_noisy, e.voiced_but_noisy, "frame {i} noisy");
            assert_eq!(m.vibrato, e.vibrato, "frame {i} vibrato");
            assert_eq!(
                m.steadiness_cents, e.steadiness_cents,
                "frame {i} steadiness"
            );
            voiced_frames += usize::from(got.valid);
        }
        assert!(
            voiced_frames >= 50,
            "the vowel must be voiced: {voiced_frames}"
        );
        let _ = AudioFrame::preallocated(1, SR);
    }
}
