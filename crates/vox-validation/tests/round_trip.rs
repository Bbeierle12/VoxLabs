//! Phase 2 gate, host side: a recorded run validates against itself
//! (provenance round-trip within the D5 bands), and the Fingerprint mode
//! reproduces the analyzer's enrollment features on a synthetic fixture.

use std::f32::consts::TAU;
use std::path::PathBuf;

use vox_core::config::ToleranceConfig;
use vox_core::frame::{ANALYSIS_FRAME, FrameAnalyzer};
use vox_core::math;
use vox_core::pipeline::PipelineDefinition;
use vox_core::pipeline::consumers::profile_from_wires;
use vox_core::pipeline::offline::run_offline;
use vox_core::types::{Formant, N_FORMANTS};
use vox_validation::{record_run, validate_and_write};

const SR: u32 = 48_000;

fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "vox-validation-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A harmonic vowel with mild vibrato, quiet lead-in and tail.
fn vowel(f0: f32, secs: f32) -> Vec<f32> {
    let n = (SR as f32 * secs) as usize;
    let lead = ANALYSIS_FRAME * 20;
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    let mut noise = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        0.001 * ((seed >> 40) as f32 / (1u64 << 24) as f32 - 0.5)
    };
    let mut v: Vec<f32> = (0..lead).map(|_| noise()).collect();
    v.extend((0..n).map(|i| {
        let t = i as f32 / SR as f32;
        let f = f0 * (1.0 + 0.008 * (TAU * 5.5 * t).sin());
        (1i32..=12)
            .map(|k| (0.62f32).powi(k - 1) * (TAU * f * k as f32 * t).sin())
            .sum::<f32>()
            * 0.18
    }));
    v.extend(std::iter::repeat_n(0.0, ANALYSIS_FRAME * 4));
    v
}

fn write_wav(path: &std::path::Path, samples: &[f32]) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: SR,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(path, spec).unwrap();
    for &s in samples {
        w.write_sample((s.clamp(-1.0, 1.0) * 32767.0) as i16)
            .unwrap();
    }
    w.finalize().unwrap();
}

fn coarse_live_model(dir: &std::path::Path) -> PathBuf {
    let p = dir.join("live_model_coarse.toml");
    std::fs::write(
        &p,
        format!(
            "{}\n[params.tract]\ngrid_n = 21\n",
            PipelineDefinition::LIVE_MODEL
        ),
    )
    .unwrap();
    p
}

#[test]
fn a_recorded_run_round_trips_within_the_bands() {
    let dir = temp_dir("roundtrip");
    let wav = dir.join("vowel_a_140.wav");
    write_wav(&wav, &vowel(140.0, 2.0));
    let mode_path = coarse_live_model(&dir);
    let mode = PipelineDefinition::by_name_or_path(mode_path.to_str().unwrap()).unwrap();
    let out = dir.join("results");
    let (hops, provenance) = record_run(&mode, &wav, &out, SR as f32).unwrap();
    assert!(hops > 60, "{hops} hops");
    assert!(out.join("vowel_a_140.taps.jsonl").is_file());

    let report = validate_and_write(&provenance).unwrap();
    assert!(report.input_digest_matched);
    assert!(report.params_digest_matched);
    assert_eq!(report.recorded_hops, hops);
    assert_eq!(report.rerun_hops, hops);
    assert!(
        report.pass,
        "round-trip outside bands: {:#?}",
        report.compare.per_type
    );
    assert!(report.compare.per_type.contains_key("AreaFunction"));
    assert!(out.join("vowel_a_140.validation.json").is_file());
    let evidence: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(out.join("vowel_a_140.evidence.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(evidence["banner"], "NOT VALIDATED");
    let gates = evidence["gates"].as_array().unwrap();
    let rt = gates
        .iter()
        .find(|g| g["name"] == "Provenance round-trip within tolerance bands")
        .unwrap();
    assert_eq!(rt["met"], true);

    // A changed input is caught: the digest no longer matches.
    write_wav(&wav, &vowel(150.0, 2.0));
    let report = validate_and_write(&provenance).unwrap();
    assert!(!report.input_digest_matched);
    assert!(!report.pass);
    let _ = std::fs::remove_dir_all(dir);
}

/// The plan's Phase 2 gate: Fingerprint mode reproduces the analyzer's
/// enrollment features on a synthetic fixture within tolerance. Frame k
/// of the analyzer is hop 2k of the pipeline (hop 1024, frame 2048).
#[test]
fn fingerprint_mode_reproduces_the_analyzers_enrollment_features() {
    vox_core::room::reset();
    let signal = vowel(130.0, 4.0);
    let sr = SR as f32;
    let tol = ToleranceConfig::DEFAULT;

    // Reference: the analyzer, frame by frame.
    let mut analyzer = FrameAnalyzer::new(sr);
    let reference: Vec<_> = signal
        .chunks_exact(ANALYSIS_FRAME)
        .map(|f| analyzer.analyze(f).profile)
        .collect();

    // Pipeline: every hop; keep the even ones.
    let mode = PipelineDefinition::by_name_or_path("fingerprint").unwrap();
    let mut pipeline_profiles = Vec::new();
    run_offline(&mode, sr, &signal, |h| {
        if h.hop % 2 == 0 {
            pipeline_profiles.push(profile_from_wires(h.wires).unwrap());
        }
    })
    .unwrap();
    assert_eq!(pipeline_profiles.len(), reference.len());

    let mut voiced_agree = 0;
    let mut voiced_total = 0;
    let mut id_ref: [Vec<f32>; N_FORMANTS] = Default::default();
    let mut id_pipe: [Vec<f32>; N_FORMANTS] = Default::default();
    for (k, (r, p)) in reference.iter().zip(&pipeline_profiles).enumerate() {
        if r.valid || p.valid {
            voiced_total += 1;
            voiced_agree += usize::from(r.valid == p.valid);
        }
        if r.valid && p.valid {
            assert!(
                (r.f0 - p.f0).abs() <= tol.f0_hz,
                "frame {k}: f0 {} vs {}",
                r.f0,
                p.f0
            );
            if r.formants_f0 == r.f0 && p.formants_f0 == p.f0 {
                for i in 0..N_FORMANTS {
                    assert!(
                        (r.formants[i].frequency - p.formants[i].frequency).abs() <= tol.formant_hz,
                        "frame {k}: F{} {} vs {}",
                        i + 1,
                        r.formants[i].frequency,
                        p.formants[i].frequency
                    );
                }
            }
            let grade = math::formant_grade(&r.formants, r.formants_f0);
            if grade == math::FormantGrade::Identity && r.formants_f0 == r.f0 {
                for i in 0..N_FORMANTS {
                    id_ref[i].push(r.formants[i].frequency);
                    id_pipe[i].push(p.formants[i].frequency);
                }
            }
        }
    }
    assert!(voiced_total > 40, "{voiced_total} voiced frames");
    assert!(
        voiced_agree as f32 / voiced_total as f32 >= 0.95,
        "voicing agreement {voiced_agree}/{voiced_total}"
    );
    assert!(
        id_ref[0].len() > 20,
        "identity-grade frames: {}",
        id_ref[0].len()
    );

    // Enrollment: the median identity formants of each path build the
    // voiceprint; the two prints must match as one voice.
    let median = |v: &mut Vec<f32>| {
        v.sort_by(f32::total_cmp);
        v[v.len() / 2]
    };
    let print = |acc: &mut [Vec<f32>; N_FORMANTS]| {
        let f: [Formant; N_FORMANTS] = std::array::from_fn(|i| Formant {
            frequency: median(&mut acc[i]),
            bandwidth: 0.0,
        });
        math::build_voiceprint(Some(f), &[1.0; 16], None, None)
    };
    let a = print(&mut id_ref);
    let b = print(&mut id_pipe);
    for i in 0..N_FORMANTS {
        assert!(
            (a.formants[i] - b.formants[i]).abs() <= tol.formant_hz,
            "enrolled F{}: {} vs {}",
            i + 1,
            a.formants[i],
            b.formants[i]
        );
    }
    let similarity = math::voiceprint_similarity(&a, &b).expect("scorable");
    assert!(similarity >= 99.0, "match {similarity}");
}
