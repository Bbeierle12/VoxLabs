//! Coral's `harmony.test.ts` (targets, profiles, anchoring, drift), on the port.

use super::*;
use crate::choir::harmony::midi_to_hz;
use crate::choir::harmony::*;
use crate::choir::labeler::{Label, Section};
use crate::config::ChoirHarmonyConfig;

const A4: f32 = 440.0;

fn v(section: Section, pc: i32, oct: i32, c: f32) -> VoiceIn {
    let f0 = midi_to_hz((12 * (oct + 1) + pc) as f32, A4) * 2f32.powf(c / 1200.0);
    VoiceIn {
        midi: note_info(f0, A4).midi,
        f0_hz: f0,
        salience: 2.0,
        section,
        label: Label::Section(section),
        section_conf: 0.8,
    }
}

fn run(h: &mut HarmonyAnalyzer, voices: &[VoiceIn], frames: usize, t0: f32) -> HarmonyResult {
    let mut r = h.analyze(voices, None, 44_100.0 / 8192.0, -120.0, t0);
    for i in 1..frames {
        r = h.analyze(
            voices,
            None,
            44_100.0 / 8192.0,
            -120.0,
            t0 + i as f32 * 47.0,
        );
    }
    r
}

fn cmaj_pure() -> Vec<VoiceIn> {
    vec![
        v(Section::B, 0, 3, 0.0),
        v(Section::T, 7, 3, 2.0),
        v(Section::A, 4, 4, -13.7),
        v(Section::S, 0, 5, 0.0),
    ]
}

fn analyzer() -> HarmonyAnalyzer {
    HarmonyAnalyzer::new(HarmonyConfig::default(), ChoirHarmonyConfig::DEFAULT)
}

/// Coral's `harmony.test.ts`: targets, profiles, anchoring.
#[test]
fn pure_c_major_reads_zero_error_and_ensemble_minus_7_7_on_the_third() {
    let mut h = analyzer();
    h.set_config(HarmonyConfig {
        profile: Profile::Pure,
        ..HarmonyConfig::default()
    });
    let r = run(&mut h, &cmaj_pure(), 12, 1000.0);
    assert_eq!(r.id.unwrap().chord.name, "Major");
    assert!(r.settled);
    for t in &r.tg {
        assert!(
            t.as_ref().map(|t| t.err.abs()).unwrap_or(99.0) < 0.5,
            "{:?}",
            r.tg
        );
    }
    h.set_config(HarmonyConfig {
        profile: Profile::Ensemble,
        ..HarmonyConfig::default()
    });
    let r2 = run(&mut h, &cmaj_pure(), 12, 1000.0);
    let alto = r2
        .voices
        .iter()
        .position(|x| x.section == Section::A)
        .unwrap();
    assert!((r2.tg[alto].as_ref().unwrap().err - -7.7).abs() < 0.5);
}

#[test]
fn anchors_on_the_bass() {
    let mut h = analyzer();
    h.set_config(HarmonyConfig {
        profile: Profile::Pure,
        ..HarmonyConfig::default()
    });
    let mut det = cmaj_pure();
    det[0].f0_hz *= 2f32.powf(10.0 / 1200.0);
    let r = run(&mut h, &det, 12, 1000.0);
    assert!(r.tg[0].as_ref().unwrap().anchor);
    assert!((r.voices[0].info.cents - 10.0).abs() < 0.1);
    for i in 1..4 {
        assert!((r.tg[i].as_ref().unwrap().err - -10.0).abs() < 0.5);
    }
}

#[test]
fn dominant_seventh_sizes_the_fifth_to_seventh_pair_as_7_to_6() {
    let mut h = analyzer();
    let r = run(
        &mut h,
        &[
            v(Section::B, 7, 2, 0.0),
            v(Section::T, 2, 4, 2.0),
            v(Section::A, 5, 4, -31.2),
            v(Section::S, 11, 4, -13.7),
        ],
        12,
        1000.0,
    );
    assert_eq!(r.id.unwrap().chord.name, "Dom 7");
    let p = r
        .pairs
        .iter()
        .find(|p| r.voices[p.a].info.name == "D" && r.voices[p.b].info.name == "F")
        .unwrap();
    assert_eq!(p.ratio, "7:6");
}

#[test]
fn settles_after_300_ms_and_reports_drift_after_flat_updates() {
    let mut h = analyzer();
    assert!(!run(&mut h, &cmaj_pure(), 4, 1000.0).settled);
    run(&mut h, &cmaj_pure(), 12, 5000.0);
    let flat: Vec<VoiceIn> = cmaj_pure()
        .into_iter()
        .map(|x| VoiceIn {
            f0_hz: x.f0_hz * 2f32.powf(-30.0 / 1200.0),
            ..x
        })
        .collect();
    let mut r = h.analyze(&flat, None, 44_100.0 / 8192.0, -120.0, 20_000.0);
    for i in 1..22 {
        r = h.analyze(
            &flat,
            None,
            44_100.0 / 8192.0,
            -120.0,
            20_000.0 + i as f32 * 2100.0,
        );
    }
    let want = -30.0 * (1.0 - 0.85f32.powi(20));
    assert!(
        (r.drift.unwrap() - want).abs() < 0.5,
        "{:?} vs {want}",
        r.drift
    );
}

#[test]
fn drops_voices_whose_section_is_off() {
    let mut h = analyzer();
    h.set_config(HarmonyConfig {
        sections: [true, true, false, false],
        ..HarmonyConfig::default()
    });
    let r = run(&mut h, &cmaj_pure(), 12, 1000.0);
    assert_eq!(
        r.voices.iter().map(|x| x.section).collect::<Vec<_>>(),
        [Section::B, Section::T]
    );
}
