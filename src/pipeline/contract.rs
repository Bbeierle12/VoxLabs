//! The Phase 1 contract checks as plain functions, so they run in two
//! places from one body: as host tests (`cargo test`), and on the phone
//! from the Engineering Console's self-test (there is no adb on the study
//! phone, so the on-device column of the contract-test report is produced
//! by the app itself). Each check returns `Ok(summary)` or `Err(why)`.

use std::f32::consts::TAU;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use super::builder::{WiringError, build};
use super::definition::PipelineDefinition;
use super::runner::{Runner, RunnerState};
use super::stage::{Stage, StreamFormat};
use super::stages::inverse::{GridInverseStage, shared_grid};
use super::stages::lpc::LpcStage;
use super::stages::tract::StoryTractStage;
use super::stages::yin::YinStage;
use super::types::{
    AreaFunction, AudioFrame, BasisId, F0Track, FormantTrack, TractParams, Wire, WireType,
};
use crate::config::PipelineParams;
use crate::frame::{ANALYSIS_FRAME, FrameAnalyzer};
use crate::math;
use crate::tract::{ADULT_MALE, area_function, resonances};
use crate::types::Formant;

/// Sample rate every check runs at.
pub const SR: f32 = 48_000.0;
/// Coarse inversion grid for the checks (tract.rs's own tests use 21×21).
pub const COARSE_GRID_N: usize = 21;
/// How long the runner check waits for the worker to build and to finish.
const RUNNER_WAIT: Duration = Duration::from_secs(60);
const RUNNER_POLL: Duration = Duration::from_millis(5);
/// Odd push sizes: cpal #902, the callback's chunking is not to be trusted.
const CHUNKS: [usize; 7] = [7, 480, 1, 1023, 3000, 512, 1025];

/// The Live Model definition with the coarse grid — an override through
/// the documented `[params]` mechanism, so that path is exercised too.
pub fn live_model_coarse() -> Result<PipelineDefinition, String> {
    let text = format!(
        "{}\n[params.tract]\ngrid_n = {COARSE_GRID_N}\n",
        PipelineDefinition::LIVE_MODEL
    );
    PipelineDefinition::from_toml(&text).map_err(|e| e.to_string())
}

fn fmt() -> StreamFormat {
    StreamFormat {
        sample_rate_hz: SR,
        frame_samples: ANALYSIS_FRAME,
        hop: ANALYSIS_FRAME / 2,
    }
}

/// Harmonic-rich vowel at `f0`, filtered by the Story tract at (q1, q2)
/// so the formants are physical and the inverse can find them.
pub fn vowel(f0: f32, q1: f32, q2: f32, n: usize) -> Vec<f32> {
    let areas = area_function(&ADULT_MALE, q1, q2);
    let [r1, r2, r3] = resonances(&areas, ADULT_MALE.vtl_cm).expect("resonances");
    let gain = |f: f32| -> f32 {
        let peak = |r: f32| 1.0 / (1.0 + ((f - r) / 40.0).powi(2));
        (peak(r1) + peak(r2) + peak(r3) + 0.05) / (1.0 + f / 500.0)
    };
    (0..n)
        .map(|i| {
            let t = i as f32 / SR;
            (1..=30)
                .map(|k| {
                    let fk = f0 * k as f32;
                    if fk >= SR / 2.0 {
                        0.0
                    } else {
                        gain(fk) * (TAU * fk * t).sin()
                    }
                })
                .sum::<f32>()
                * 0.15
        })
        .collect()
}

fn sine(f0: f32, n: usize) -> Vec<f32> {
    (0..n).map(|i| (TAU * f0 * i as f32 / SR).sin()).collect()
}

fn sawtooth_like(f0: f32, n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| {
            let t = i as f32 / SR;
            (1i32..=10)
                .map(|k| (0.6f32).powi(k - 1) * (TAU * f0 * k as f32 * t).sin())
                .sum::<f32>()
                * 0.2
        })
        .collect()
}

fn check<T: PartialEq + std::fmt::Debug>(what: &str, got: T, want: T) -> Result<(), String> {
    if got == want {
        Ok(())
    } else {
        Err(format!("{what}: got {got:?}, expected {want:?}"))
    }
}

/// Every check, in order, with its code.
pub fn run_all() -> Vec<(&'static str, Result<String, String>)> {
    vec![
        ("definition_builds", definition_builds()),
        ("wrong_wiring_names_adapters", wrong_wiring_names_adapters()),
        ("yin_matches_direct", yin_matches_direct()),
        ("lpc_matches_frame_analyzer", lpc_matches_frame_analyzer()),
        ("inverse_round_trips_vowels", inverse_round_trips_vowels()),
        ("tract_matches_area_function", tract_matches_area_function()),
        (
            "runner_reframes_and_tube_moves",
            runner_reframes_and_tube_moves(),
        ),
    ]
}

pub fn definition_builds() -> Result<String, String> {
    let def = PipelineDefinition::live_model().map_err(|e| e.to_string())?;
    check("hop (D14)", def.format.hop, 1024)?;
    let coarse = live_model_coarse()?;
    let n = coarse.stages.len();
    let p = build(&coarse, coarse.format(Some(SR))).map_err(|e| e.to_string())?;
    check("stages", p.stages.len(), n)?;
    check("wires", p.wires.len(), n + 1)?;
    check("taps", p.taps.clone(), (0..n).collect::<Vec<_>>())?;
    let types: Vec<WireType> = p.wires.iter().map(Wire::wire_type).collect();
    check(
        "wire types",
        types,
        vec![
            WireType::AudioFrame,
            WireType::F0Track,
            WireType::F0Track,
            WireType::FormantTrack,
            WireType::HarmonicSeries,
            WireType::VoiceMetrics,
            WireType::VoiceMetrics,
            WireType::TractParams,
            WireType::AreaFunction,
            WireType::Spectrum,
        ],
    )?;
    Ok(format!(
        "{}: {} stages, {} wires preallocated",
        def.name,
        p.stages.len(),
        p.wires.len()
    ))
}

pub fn wrong_wiring_names_adapters() -> Result<String, String> {
    let mut def = live_model_coarse()?;
    def.stages
        .retain(|s| s.name != "lpc" && s.name != "inverse");
    def.taps = vec!["yin".into(), "tract".into()];
    let err = match build(&def, def.format(Some(SR))) {
        Ok(_) => return Err("yin → tract built; it must be refused".into()),
        Err(e) => e,
    };
    let WiringError::MissingInput {
        stage,
        needed,
        adapter_chain,
        ..
    } = &err
    else {
        return Err(format!("wrong error kind: {err}"));
    };
    check("stage", stage.as_str(), "tract")?;
    check("needed", *needed, WireType::TractParams)?;
    let chain: Vec<&str> = adapter_chain.iter().map(|a| a.name).collect();
    check("adapter chain", chain, vec!["lpc", "inverse"])?;
    Ok(format!("refused at build; adapters named: {}", {
        let names: Vec<&str> = adapter_chain.iter().map(|a| a.name).collect();
        names.join(" → ")
    }))
}

pub fn yin_matches_direct() -> Result<String, String> {
    let mut s = YinStage::init(&PipelineParams::DEFAULT, &fmt()).map_err(|e| e.to_string())?;
    let frame = AudioFrame {
        samples: sine(220.0, ANALYSIS_FRAME),
        sample_rate: SR,
        frame_index: 0,
    };
    let mut out = F0Track::default();
    s.process(&frame, &mut out).map_err(|e| e.to_string())?;
    let direct = math::yin_pitch(&frame.samples, SR).ok_or("direct YIN found nothing")?;
    check("hz", out.hz, direct.f0)?;
    check("confidence", out.confidence, direct.confidence)?;
    check("voiced", out.voiced, true)?;
    if (out.hz - 220.0).abs() >= 2.0 {
        return Err(format!("220 Hz fixture read {:.2} Hz", out.hz));
    }
    let silent = AudioFrame::preallocated(ANALYSIS_FRAME, SR);
    s.process(&silent, &mut out).map_err(|e| e.to_string())?;
    check("silence unvoiced", out.voiced, false)?;
    Ok(format!(
        "stage == math::yin_pitch ({:.2} Hz); silence unvoiced",
        direct.f0
    ))
}

pub fn lpc_matches_frame_analyzer() -> Result<String, String> {
    let sig = sawtooth_like(140.0, ANALYSIS_FRAME * 4);
    let mut analyzer = FrameAnalyzer::new(SR);
    let mut stage = LpcStage::init(&PipelineParams::DEFAULT, &fmt()).map_err(|e| e.to_string())?;
    let mut yin = YinStage::init(&PipelineParams::DEFAULT, &fmt()).map_err(|e| e.to_string())?;
    let mut frames = 0;
    for (i, chunk) in sig.chunks_exact(ANALYSIS_FRAME).enumerate() {
        let expected = analyzer.analyze(chunk).profile;
        let frame = AudioFrame {
            samples: chunk.to_vec(),
            sample_rate: SR,
            frame_index: i as u64,
        };
        let mut f0 = F0Track::default();
        yin.process(&frame, &mut f0).map_err(|e| e.to_string())?;
        let mut ft = FormantTrack::held_default(&PipelineParams::DEFAULT.formants);
        stage
            .process((&frame, &f0), &mut ft)
            .map_err(|e| e.to_string())?;
        check(
            &format!("frame {i} formants"),
            ft.formants,
            expected.formants,
        )?;
        check(
            &format!("frame {i} measured_f0"),
            ft.measured_f0,
            expected.formants_f0,
        )?;
        frames += 1;
    }
    Ok(format!(
        "{frames} frames: stage formants == FrameAnalyzer formants"
    ))
}

fn track(f: [f32; 3], measured_f0: f32) -> FormantTrack {
    FormantTrack {
        formants: [
            Formant {
                frequency: f[0],
                bandwidth: 80.0,
            },
            Formant {
                frequency: f[1],
                bandwidth: 120.0,
            },
            Formant {
                frequency: f[2],
                bandwidth: 160.0,
            },
        ],
        measured_f0,
        confidence: 1.0,
        fresh: true,
    }
}

pub fn inverse_round_trips_vowels() -> Result<String, String> {
    let mut params = PipelineParams::DEFAULT;
    params.tract.grid_n = COARSE_GRID_N;
    let mut s = GridInverseStage::init(&params, &fmt()).map_err(|e| e.to_string())?;
    let f0 = F0Track {
        hz: 120.0,
        confidence: 0.9,
        voiced: true,
        ..Default::default()
    };
    let mut names = Vec::new();
    for (name, q1, q2) in [("i", -5.10, 0.88), ("ɑ", 3.86, 1.35), ("u", -3.48, -1.70)] {
        let areas = area_function(&ADULT_MALE, q1, q2);
        let [r1, r2, r3] = resonances(&areas, ADULT_MALE.vtl_cm).ok_or("no resonances")?;
        let measured_f0 = [210.0f32, 230.0, 260.0, 290.0, 320.0, 345.0]
            .into_iter()
            .find(|&f| {
                math::formant_grade(&track([r1, r2, r3], f).formants, f)
                    == math::FormantGrade::DisplayOnly
            })
            .ok_or("no display-grade f0 clear of the suspect bands")?;
        let ft = track([r1, r2, r3], measured_f0);
        let mut out = TractParams::default();
        s.process((&f0, &ft), &mut out).map_err(|e| e.to_string())?;
        let direct = shared_grid(&ADULT_MALE, COARSE_GRID_N)
            .invert(r1, r2)
            .ok_or("direct inversion failed")?;
        check(&format!("vowel {name} valid"), out.valid, true)?;
        check(&format!("vowel {name} == direct"), (out.q1, out.q2), direct)?;
        if (out.q1 - q1).abs() >= 0.6 || (out.q2 - q2).abs() >= 0.5 {
            return Err(format!(
                "vowel {name}: ({:.2}, {:.2}) vs published ({q1}, {q2})",
                out.q1, out.q2
            ));
        }
        names.push(name);
    }
    let mut out = TractParams::default();
    s.process(
        (&F0Track::default(), &track([700.0, 1200.0, 2500.0], 120.0)),
        &mut out,
    )
    .map_err(|e| e.to_string())?;
    check("unvoiced not valid", out.valid, false)?;
    Ok(format!(
        "vowels {} round-trip; unvoiced frames are not valid",
        names.join(", ")
    ))
}

pub fn tract_matches_area_function() -> Result<String, String> {
    let mut s =
        StoryTractStage::init(&PipelineParams::DEFAULT, &fmt()).map_err(|e| e.to_string())?;
    let mut p = TractParams {
        q1: -5.10,
        q2: 0.88,
        uncertainty: None,
        valid: true,
        basis: BasisId::AdultMale,
        vtl_est_cm: None,
    };
    let mut out = AreaFunction::neutral(BasisId::AdultMale);
    s.process(&p, &mut out).map_err(|e| e.to_string())?;
    check(
        "diameters",
        out.diameters_cm,
        crate::tract::diameters(&ADULT_MALE, -5.10, 0.88),
    )?;
    check(
        "areas",
        out.areas_cm2,
        area_function(&ADULT_MALE, -5.10, 0.88),
    )?;
    check("live", out.live, true)?;
    let shape_i = out.diameters_cm;
    p.q1 = 3.86;
    p.q2 = 1.35;
    s.process(&p, &mut out).map_err(|e| e.to_string())?;
    if out.diameters_cm == shape_i {
        return Err("two vowels gave one shape".into());
    }
    let shape_a = out.diameters_cm;
    p.valid = false;
    s.process(&p, &mut out).map_err(|e| e.to_string())?;
    check("held shape", out.diameters_cm, shape_a)?;
    check("held not live", out.live, false)?;
    Ok("stage == tract::area_function; the tube moves; invalid frames hold".into())
}

pub fn runner_reframes_and_tube_moves() -> Result<String, String> {
    let def = live_model_coarse()?;
    let format = def.format(Some(SR));
    let (mut tx, rx) = rtrb::RingBuffer::<f32>::new(1 << 16);
    let shutdown = Arc::new(AtomicBool::new(false));
    let (mut runner, shell) = Runner::spawn(def, format, rx, None, shutdown);
    let state = runner.wait_ready(RUNNER_WAIT, RUNNER_POLL);
    if state != RunnerState::Running {
        return Err(format!("runner did not start: {state:?}"));
    }
    let mut msgs = Vec::new();
    let mut signal = vowel(120.0, -5.10, 0.88, SR as usize);
    signal.extend(vowel(120.0, 3.86, 1.35, SR as usize));
    let total = signal.len();
    let mut offset = 0;
    let mut k = 0;
    while offset < total {
        let n = CHUNKS[k % CHUNKS.len()].min(total - offset);
        for &s in &signal[offset..offset + n] {
            while tx.push(s).is_err() {
                msgs.extend(shell.taps.drain());
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        msgs.extend(shell.taps.drain());
        offset += n;
        k += 1;
    }
    let expected_hops = ((total - format.frame_samples) / format.hop + 1) as u64;
    let deadline = Instant::now() + RUNNER_WAIT;
    while shell.stats.hops.load(Ordering::Relaxed) < expected_hops && Instant::now() < deadline {
        msgs.extend(shell.taps.drain());
        std::thread::sleep(RUNNER_POLL);
    }
    let hops = shell.stats.hops.load(Ordering::Relaxed);
    runner.stop();
    msgs.extend(shell.taps.drain());
    check("hops = floor((n - frame) / hop) + 1", hops, expected_hops)?;
    check(
        "stage errors",
        shell.stats.stage_errors.load(Ordering::Relaxed),
        0,
    )?;
    check(
        "tap drops",
        shell.stats.tap_drops.load(Ordering::Relaxed),
        0,
    )?;
    let n_taps = shell.stages.len() as u64;
    check("taps per hop", msgs.len() as u64, expected_hops * n_taps)?;
    let mid_i = (SR as usize / format.hop / 2) as u64;
    let mid_a = expected_hops - mid_i;
    let at = |hop: u64, stage: &str| -> Result<&Wire, String> {
        msgs.iter()
            .find(|m| m.hop == hop && &*m.stage_name == stage)
            .map(|m| &m.value)
            .ok_or_else(|| format!("no tap for hop {hop} stage {stage}"))
    };
    let Wire::F0Track(f0) = at(mid_i, "voicing")? else {
        return Err("voicing tap is not F0Track".into());
    };
    if !(f0.voiced && (f0.hz - 120.0).abs() < 3.0) {
        return Err(format!("f0 tap on /i/: {f0:?}"));
    }
    let (Wire::TractParams(tp_i), Wire::TractParams(tp_a)) =
        (at(mid_i, "inverse")?, at(mid_a, "inverse")?)
    else {
        return Err("inverse tap is not TractParams".into());
    };
    if !(tp_i.valid && tp_a.valid && tp_a.q1 > tp_i.q1) {
        return Err(format!("inverse on the two vowels: {tp_i:?} vs {tp_a:?}"));
    }
    let (Wire::AreaFunction(af_i), Wire::AreaFunction(af_a)) =
        (at(mid_i, "tract")?, at(mid_a, "tract")?)
    else {
        return Err("tract tap is not AreaFunction".into());
    };
    if !(af_i.live && af_a.live) || af_i.diameters_cm == af_a.diameters_cm {
        return Err("the tube did not move between the vowels".into());
    }
    Ok(format!(
        "{hops} hops from odd-sized pushes; 4 taps each; worst hop {} µs of {} µs",
        shell.stats.worst_hop_us.load(Ordering::Relaxed),
        shell.stats.hop_budget_us.load(Ordering::Relaxed)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The on-device self-test body, run on the host too.
    #[test]
    fn every_contract_check_passes() {
        for (code, result) in run_all() {
            assert!(result.is_ok(), "{code}: {}", result.unwrap_err());
        }
    }
}
