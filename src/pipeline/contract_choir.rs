//! The Phase 4 contract checks (Coral's choir branch), run as host tests
//! and from the phone's self-test through `contract::run_all`.

use super::contract::{SR, check};
use super::definition::PipelineDefinition;
use super::types::Wire;
use crate::config::PipelineParams;

/// Phase 4: Coral's STFT reproduces librosa on the 48 kHz oracle fixture
/// within Coral's cross-implementation tier (rtol 1e-4, atol 1e-7, bin 0
/// excluded as Coral excludes it).
pub fn choir_stft_matches_librosa() -> Result<String, String> {
    use crate::choir::stft::CoralStft;
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Oracle {
        fft_size: usize,
        hop_size: usize,
        num_bins: usize,
        frames: usize,
        samples: Vec<f32>,
        expected_magnitudes: Vec<Vec<f32>>,
    }
    const TEXT: &str =
        include_str!("../../apps/coral/src/audio/__fixtures__/oracle/sine-bin100-48k-2048.json");
    let fx: Oracle = serde_json::from_str(TEXT).map_err(|e| e.to_string())?;
    let mut stft = CoralStft::new(fx.fft_size)?;
    let mut got = vec![0.0f32; fx.num_bins];
    let (rtol, atol) = (1e-4f32, 1e-7f32);
    let mut max_rel = 0.0f32;
    let mut frames = 0;
    let mut start = 0;
    while start + fx.fft_size <= fx.samples.len() {
        stft.magnitudes(&fx.samples[start..start + fx.fft_size], &mut got)?;
        let exp = &fx.expected_magnitudes[frames];
        for b in 1..fx.num_bins {
            let err = (got[b] - exp[b]).abs();
            if err > atol + rtol * exp[b].abs() {
                return Err(format!(
                    "frame {frames} bin {b}: got {}, librosa {}",
                    got[b], exp[b]
                ));
            }
            if exp[b].abs() > atol / rtol {
                max_rel = max_rel.max(err / exp[b].abs());
            }
        }
        frames += 1;
        start += fx.hop_size;
    }
    check("frames", frames, fx.frames)?;
    Ok(format!(
        "{frames} frames · max rel {max_rel:.2e} (tier 1e-4)"
    ))
}

fn choir_last_frame(
    voices: &[(crate::choir::labeler::Section, i32)],
    secs: f64,
    seed: u32,
    sr: f32,
    one_clean_singer_per_part: bool,
) -> Result<
    (
        crate::pipeline::types::NoteSet,
        crate::choir::cards::HarmonyResult,
    ),
    String,
> {
    use super::offline::run_offline;
    use crate::choir::harmony::HarmonyConfig;
    use crate::choir::pipeline::ChoirHarmony;
    use crate::choir::synth::{ChordSpec, VoiceSpec, synthesize_choir_chord};
    let mut spec = ChordSpec::new(
        f64::from(sr),
        secs,
        voices
            .iter()
            .map(|&(s, m)| {
                let mut v = VoiceSpec::new(s, m);
                if one_clean_singer_per_part {
                    v.singers = 1;
                    v.detune_cents = 0.0;
                    v.vibrato_cents = 0.0;
                }
                v
            })
            .collect(),
    );
    spec.seed = seed;
    let samples = synthesize_choir_chord(&spec)?.samples;
    let def = PipelineDefinition::by_name_or_path("choir").map_err(|e| e.to_string())?;
    let p = PipelineParams::DEFAULT;
    let mut harmony = ChoirHarmony::new(HarmonyConfig::default(), p.choir_harmony, p.choir_stft);
    let (hop, frame) = (def.format.hop, def.format.frame_samples);
    let mut last = None;
    run_offline(&def, sr, &samples, |h| {
        if let (Some(Wire::Spectrum(s)), Some(Wire::NoteSet(n)), Some(Wire::SectionLabels(l))) =
            (h.wires.get(1), h.wires.get(3), h.wires.get(4))
        {
            let now_ms = (h.hop as usize * hop + frame) as f32 / sr * 1000.0;
            let r = harmony.feed(s, n, l, now_ms);
            last = Some((n.clone(), r));
        }
    })
    .map_err(|e| e.to_string())?;
    last.ok_or_else(|| "the choir mode produced no frames".into())
}

/// Phase 4: Coral's calibrated contract — a close-voiced C–E–G triad is
/// recovered exactly by the multi-F0 stage.
pub fn choir_detects_a_close_triad() -> Result<String, String> {
    use crate::choir::labeler::Section;
    let (notes, _) = choir_last_frame(
        &[(Section::B, 60), (Section::T, 64), (Section::A, 67)],
        0.4,
        7,
        SR,
        false,
    )?;
    check("active notes", notes.active_midi.clone(), vec![60, 64, 67])?;
    Ok(format!(
        "{:?} · voices {:?}",
        notes.active_midi,
        notes.voices.iter().map(|v| v.midi).collect::<Vec<_>>()
    ))
}

/// Phase 4: the rehearsal harmony over the taps hears a settled C major
/// with four voices on their sections (one clean singer per part, as
/// Coral's own through-the-pipeline test sings it).
pub fn choir_harmony_hears_c_major() -> Result<String, String> {
    use crate::choir::labeler::Section;
    let (_, r) = choir_last_frame(
        &[
            (Section::B, 48),
            (Section::T, 55),
            (Section::A, 64),
            (Section::S, 72),
        ],
        1.5,
        3,
        SR,
        true,
    )?;
    let id = r.id.ok_or("no chord")?;
    check("chord", (id.chord.name, id.root), ("Major", 0))?;
    check("settled", r.settled, true)?;
    check(
        "voices",
        r.voices.iter().map(|v| v.info.midi).collect::<Vec<_>>(),
        vec![48, 55, 64, 72],
    )?;
    Ok(format!(
        "{} {} · consonance {:?} · held {:.0} ms · {}",
        crate::choir::harmony::NOTE_NAMES[id.root],
        id.chord.name,
        r.cons,
        r.chord_held_ms,
        r.voices
            .iter()
            .map(|v| format!(
                "{}{} {:+.1}c",
                v.section.letter(),
                v.info.midi,
                v.info.cents
            ))
            .collect::<Vec<_>>()
            .join(" ")
    ))
}
