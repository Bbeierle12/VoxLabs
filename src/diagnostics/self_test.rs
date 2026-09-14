//! The console's "Run self-test": nine checks a phone can run on itself
//! with no adb. The source app's set (reduced model, lumen asset, shared
//! model contract, synthesis math, audio record format, private storage,
//! pitch continuity, stationary noise, microphone permission) mapped onto
//! what VoxLabs ships: its mode file, its inversion grids, its config
//! contract, its oscillator bank, its cpal input, its data folder, its YIN,
//! its noise floor, and the same permission check.

use std::f32::consts::TAU;
use std::path::Path;

use super::core::{CFG, SelfTestResult};
use crate::config::PipelineParams;
use crate::frame::ANALYSIS_FRAME;
use crate::pipeline::PipelineDefinition;

pub fn run(files_dir: Option<&Path>) -> Vec<SelfTestResult> {
    let mut out = Vec::new();
    let mut test = |code: &str, f: &dyn Fn() -> Result<String, String>| {
        let (passed, message) = match f() {
            Ok(m) => (true, m),
            Err(e) => (false, e),
        };
        out.push(SelfTestResult {
            code: code.into(),
            passed,
            message,
        });
    };
    test("pipeline_definition", &pipeline_definition);
    test("inversion_grids", &inversion_grids);
    test("config_contract", &config_contract);
    test("synthesis_math", &synthesis_math);
    test("audio_input_format", &audio_input_format);
    test("private_storage", &|| private_storage(files_dir));
    test("pitch_estimator", &pitch_estimator);
    test("noise_floor", &noise_floor);
    for (code, result) in crate::pipeline::contract::run_all() {
        let (passed, message) = match result {
            Ok(m) => (true, m),
            Err(e) => (false, e),
        };
        out.push(SelfTestResult {
            code: format!("contract.{code}"),
            passed,
            message,
        });
    }
    let granted = microphone_granted();
    out.push(SelfTestResult {
        code: "microphone_permission".into(),
        passed: granted,
        message: if granted {
            "Microphone permission granted".into()
        } else {
            "Permission not granted; offline tools remain usable".into()
        },
    });
    out
}

fn pipeline_definition() -> Result<String, String> {
    let def = PipelineDefinition::live_model().map_err(|e| e.to_string())?;
    if def.format.frame_samples != ANALYSIS_FRAME {
        return Err(format!(
            "Expected a {ANALYSIS_FRAME}-sample frame, got {}",
            def.format.frame_samples
        ));
    }
    Ok(format!(
        "{}; {} stages; frame {} hop {}",
        def.name,
        def.stages.len(),
        def.format.frame_samples,
        def.format.hop
    ))
}

fn inversion_grids() -> Result<String, String> {
    use crate::pipeline::stages::inverse::shared_grid;
    use crate::tract::{ADULT_FEMALE, ADULT_MALE, GRID_N};
    let n = GRID_N;
    for basis in [&ADULT_MALE, &ADULT_FEMALE] {
        let grid = shared_grid(basis, n);
        // A neutral shape must invert to itself, within a cell.
        let areas = crate::tract::area_function(basis, 0.0, 0.0);
        let res = crate::tract::resonances(&areas, basis.vtl_cm)
            .ok_or("Neutral shape has no resonances")?;
        let (q1, q2) = grid
            .invert(res[0], res[1])
            .ok_or("Neutral resonances did not invert")?;
        if q1.abs() > 1.0 || q2.abs() > 1.0 {
            return Err(format!(
                "Neutral shape inverted to ({q1:.2}, {q2:.2}), expected near (0, 0)"
            ));
        }
    }
    Ok(format!(
        "{n}×{n} grids built for adult male and adult female; neutral shape round-trips"
    ))
}

fn config_contract() -> Result<String, String> {
    let parsed = PipelineParams::from_toml(PipelineParams::TOML).map_err(|e| e.to_string())?;
    if parsed != PipelineParams::DEFAULT {
        return Err("pipeline.toml drifted from the code defaults".into());
    }
    let sections = crate::config::all_sections().len();
    Ok(format!(
        "{sections} sections; pipeline.toml matches code defaults"
    ))
}

fn synthesis_math() -> Result<String, String> {
    let mut bank = crate::synthesis::OscillatorBank::new(
        CFG.self_test_synth_rate_hz,
        CFG.self_test_synth_glide_ms,
    );
    let mut heard = false;
    for _ in 0..CFG.self_test_synth_samples {
        let (l, r) = bank.process_sample();
        if !(l.is_finite() && r.is_finite()) {
            return Err("Synthesizer fixture was silent or non-finite".into());
        }
        if l.abs() > CFG.self_test_synth_silence {
            heard = true;
        }
    }
    if !heard {
        return Err("Synthesizer fixture was silent or non-finite".into());
    }
    Ok(format!(
        "{} finite synthesis samples",
        CFG.self_test_synth_samples
    ))
}

fn audio_input_format() -> Result<String, String> {
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    let dev = host
        .default_input_device()
        .ok_or("No default input device")?;
    let name = dev
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "unnamed".into());
    let cfg = dev.default_input_config().map_err(|e| e.to_string())?;
    Ok(format!(
        "{name}; {} Hz; {} ch; {:?}; capture not started",
        cfg.sample_rate(),
        cfg.channels(),
        cfg.sample_format()
    ))
}

fn private_storage(files_dir: Option<&Path>) -> Result<String, String> {
    let dir = files_dir.ok_or("No data folder on this platform")?;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let probe = dir.join(".write-probe");
    std::fs::write(&probe, b"ok")
        .map_err(|_| "Internal app storage is not writable".to_string())?;
    let _ = std::fs::remove_file(&probe);
    Ok(format!("writable: {}", dir.display()))
}

fn pitch_estimator() -> Result<String, String> {
    let rate = CFG.self_test_synth_rate_hz;
    let tone = CFG.self_test_tone_hz;
    let frame: Vec<f32> = (0..ANALYSIS_FRAME)
        .map(|i| CFG.self_test_tone_amplitude * (TAU * tone * i as f32 / rate).sin())
        .collect();
    let est = crate::math::yin_pitch(&frame, rate).ok_or("YIN found no period in the fixture")?;
    if !(CFG.self_test_tone_min_hz..=CFG.self_test_tone_max_hz).contains(&est.f0) {
        return Err(format!(
            "YIN read {:.1} Hz for a {tone:.0} Hz fixture",
            est.f0
        ));
    }
    Ok(format!(
        "{tone:.0} Hz fixture read as {:.1} Hz, confidence {:.2}",
        est.f0, est.confidence
    ))
}

fn noise_floor() -> Result<String, String> {
    let mut floor = crate::math::NoiseFloor::new();
    for _ in 0..CFG.self_test_quiet_frames {
        floor.push_unvoiced(CFG.self_test_quiet_rms);
    }
    let snr = floor
        .snr_db(CFG.self_test_loud_rms)
        .ok_or("Floor did not learn from the quiet fixture")?;
    if snr < CFG.self_test_min_snr_db {
        return Err(format!(
            "Loud fixture reads only {snr:.1} dB over the floor"
        ));
    }
    Ok(format!(
        "Quiet fixture learned; loud fixture reads {snr:.1} dB SNR"
    ))
}

fn microphone_granted() -> bool {
    #[cfg(target_os = "android")]
    {
        crate::permission::has_record_audio()
    }
    #[cfg(not(target_os = "android"))]
    {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pure_checks_pass_on_host() {
        assert!(pipeline_definition().is_ok(), "{:?}", pipeline_definition());
        assert!(config_contract().is_ok(), "{:?}", config_contract());
        assert!(synthesis_math().is_ok(), "{:?}", synthesis_math());
        assert!(pitch_estimator().is_ok(), "{:?}", pitch_estimator());
        assert!(noise_floor().is_ok(), "{:?}", noise_floor());
        let dir = std::env::temp_dir().join(format!("voxlabs-selftest-{}", std::process::id()));
        assert!(private_storage(Some(&dir)).is_ok());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn every_check_reports_a_code() {
        let results = run(None);
        assert_eq!(
            results.len(),
            9 + crate::pipeline::contract::run_all().len()
        );
        assert_eq!(results.last().unwrap().code, "microphone_permission");
    }
}
