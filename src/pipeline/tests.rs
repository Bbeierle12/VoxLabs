//! Phase 1 acceptance tests (Plan v3 §5, Phase 1 gate), host side:
//! the walking skeleton builds from `live_model.toml`, a wrong wiring fails
//! at build with the adapter chain, every tap carries a live value, the
//! tube moves between vowels, the ring is re-framed to the hop regardless
//! of push size, and per-stage timing and deadline stats are reported
//! against the thresholds in config. The on-device numbers (D18) come from
//! the same runner on the Pixel; these prove the mechanism.

use std::f32::consts::TAU;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use super::builder::{WiringError, adapter_chain, build};
use super::definition::PipelineDefinition;
use super::runner::{HopObserver, Runner, RunnerState};
use super::types::{AudioFrame, Wire, WireType};

const SR: f32 = 48_000.0;

/// The Live Model definition with the coarse test grid (tract.rs's own
/// tests use 21×21 for speed) — an override through the documented
/// `[params]` mechanism, so the test exercises that path too.
fn live_model_coarse() -> PipelineDefinition {
    let text = format!(
        "{}\n[params.tract]\ngrid_n = 21\n",
        PipelineDefinition::LIVE_MODEL
    );
    PipelineDefinition::from_toml(&text).expect("live_model.toml with a grid override")
}

/// Harmonic-rich vowel at `f0`, filtered by the Story /ɑ/ tract so the
/// formants are physical (frame.rs's synth is a plain sawtooth; here the
/// harmonics get the model's own resonances, which the inverse can find).
fn vowel(f0: f32, q1: f32, q2: f32, n: usize) -> Vec<f32> {
    use crate::tract::{ADULT_MALE, area_function, resonances};
    let areas = area_function(&ADULT_MALE, q1, q2);
    let [r1, r2, r3] = resonances(&areas, ADULT_MALE.vtl_cm).expect("resonances");
    let gain = |f: f32| -> f32 {
        // Three resonance peaks with 80 Hz bandwidth, plus -6 dB/oct source.
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

#[test]
fn live_model_definition_loads_and_builds() {
    let def = PipelineDefinition::live_model().expect("live_model.toml");
    assert_eq!(def.name, "live_model");
    assert_eq!(def.format.hop, 1024, "D14: hop 1024 is canonical");
    let names: Vec<&str> = def.stages.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["yin", "lpc", "inverse", "tract"]);
    let coarse = live_model_coarse();
    let p = build(&coarse, coarse.format(Some(SR))).expect("build");
    assert_eq!(p.stages.len(), 4);
    assert_eq!(
        p.wires.len(),
        5,
        "source + one wire per stage, preallocated"
    );
    assert_eq!(p.taps, vec![0, 1, 2, 3]);
    let types: Vec<WireType> = p.wires.iter().map(Wire::wire_type).collect();
    assert_eq!(
        types,
        [
            WireType::AudioFrame,
            WireType::F0Track,
            WireType::FormantTrack,
            WireType::TractParams,
            WireType::AreaFunction
        ]
    );
}

#[test]
fn wiring_a_wrong_type_fails_at_build_with_the_adapter_chain() {
    // yin → tract, skipping lpc and inverse: tract needs TractParams.
    let mut def = live_model_coarse();
    def.stages.remove(2);
    def.stages.remove(1);
    def.taps = vec!["yin".into(), "tract".into()];
    let err = build(&def, def.format(Some(SR)))
        .err()
        .expect("must not build");
    let WiringError::MissingInput {
        stage,
        needed,
        adapter_chain: chain,
        ..
    } = &err
    else {
        panic!("wrong error kind: {err}");
    };
    assert_eq!(stage, "tract");
    assert_eq!(*needed, WireType::TractParams);
    let chain_names: Vec<&str> = chain.iter().map(|a| a.name).collect();
    assert_eq!(
        chain_names,
        ["lpc", "inverse"],
        "the adapters to insert, in order"
    );
    let text = err.to_string();
    assert!(text.contains("needs TractParams"), "{text}");
    assert!(
        text.contains("lpc (AudioFrame + F0Track → FormantTrack)"),
        "{text}"
    );
    assert!(
        text.contains("inverse (F0Track + FormantTrack → TractParams)"),
        "{text}"
    );
    // And a type nothing can produce says so instead of inventing a chain.
    assert!(adapter_chain(&[WireType::AudioFrame], WireType::Loudness).is_empty());
}

#[test]
fn unknown_backends_taps_and_param_keys_fail_loud() {
    let mut def = live_model_coarse();
    def.stages[0].backend = "yin_gpu".into();
    let err = build(&def, def.format(Some(SR))).err().unwrap();
    assert!(matches!(err, WiringError::UnknownBackend { .. }), "{err}");
    assert!(
        err.to_string().contains("yin_cpu"),
        "lists the known backends: {err}"
    );

    let mut def = live_model_coarse();
    def.taps.push("spectrum".into());
    let err = build(&def, def.format(Some(SR))).err().unwrap();
    assert!(matches!(err, WiringError::UnknownTap { .. }), "{err}");

    let text = format!(
        "{}\n[params.tract]\ngrid_size = 21\n",
        PipelineDefinition::LIVE_MODEL
    );
    let def = PipelineDefinition::from_toml(&text).unwrap();
    let err = def.params().err().expect("unknown key must fail");
    assert!(err.to_string().contains("grid_size"), "{err}");

    // A runner limit missing from the mode file is a load error, not a default.
    let text = PipelineDefinition::LIVE_MODEL.replace("tap_capacity = 64\n", "");
    assert!(PipelineDefinition::from_toml(&text).is_err());
}

/// Counts observer calls and records which hops it saw.
struct CountingObserver(Arc<std::sync::Mutex<Vec<u64>>>);
impl HopObserver for CountingObserver {
    fn on_hop(&mut self, frame: &AudioFrame, wires: &[Wire], hop: u64) {
        assert_eq!(frame.frame_index, hop);
        assert_eq!(wires.len(), 5);
        self.0.lock().unwrap().push(hop);
    }
}

#[test]
fn runner_reframes_to_hop_produces_every_tap_and_the_tube_moves() {
    let def = live_model_coarse();
    let format = def.format(Some(SR));
    let (mut tx, rx) = rtrb::RingBuffer::<f32>::new(1 << 16);
    let hops_seen = Arc::new(std::sync::Mutex::new(Vec::new()));
    let shutdown = Arc::new(AtomicBool::new(false));
    let (mut runner, shell) = Runner::spawn(
        def.clone(),
        format,
        rx,
        Some(Box::new(CountingObserver(hops_seen.clone()))),
        shutdown.clone(),
    );
    assert_eq!(
        runner.wait_ready(Duration::from_secs(60), Duration::from_millis(5)),
        RunnerState::Running
    );

    // /i/ then /ɑ/ at 120 Hz, one second each, pushed in odd-sized chunks
    // (cpal #902: the callback's chunking is not to be trusted). The tap
    // channel is bounded (tap_capacity), so a shell that never drained it
    // would see drops by design: drain as the runner works.
    let mut msgs = Vec::new();
    let mut signal = vowel(120.0, -5.10, 0.88, SR as usize);
    signal.extend(vowel(120.0, 3.86, 1.35, SR as usize));
    let total = signal.len();
    let mut offset = 0;
    let chunks = [7usize, 480, 1, 1023, 3000, 512, 1025];
    let mut k = 0;
    while offset < total {
        let n = chunks[k % chunks.len()].min(total - offset);
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
    let deadline = Instant::now() + Duration::from_secs(60);
    while shell.stats.hops.load(Ordering::Relaxed) < expected_hops && Instant::now() < deadline {
        msgs.extend(shell.taps.drain());
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(
        shell.stats.hops.load(Ordering::Relaxed),
        expected_hops,
        "hops = floor((n - frame) / hop) + 1, whatever the push sizes"
    );
    assert_eq!(shell.stats.stage_errors.load(Ordering::Relaxed), 0);
    runner.stop();
    assert_eq!(shell.stats.state(), RunnerState::Stopped);

    // Every tap carried a live value on every hop, none dropped.
    msgs.extend(shell.taps.drain());
    assert_eq!(shell.stats.tap_drops.load(Ordering::Relaxed), 0);
    assert_eq!(msgs.len() as u64, expected_hops * 4, "4 taps per hop");
    let seen: Vec<u64> = hops_seen.lock().unwrap().clone();
    assert_eq!(seen.len() as u64, expected_hops, "observer ran every hop");

    // f0 tap: the vowel's pitch. Formant tap: fresh measurements. Inverse:
    // valid on the steady vowel. Tract: the tube moved between the vowels.
    let mid_i = (SR as usize / format.hop / 2) as u64; // half a second into /i/
    let mid_a = expected_hops - mid_i;
    let at = |hop: u64, stage: usize| -> &Wire {
        &msgs
            .iter()
            .find(|m| m.hop == hop && m.stage == stage)
            .expect("tap present")
            .value
    };
    let Wire::F0Track(f0) = at(mid_i, 0) else {
        panic!()
    };
    assert!(f0.voiced && (f0.hz - 120.0).abs() < 3.0, "{f0:?}");
    let Wire::FormantTrack(ft) = at(mid_i, 1) else {
        panic!()
    };
    assert!(ft.fresh && ft.formants[0].frequency > 0.0, "{ft:?}");
    let Wire::TractParams(tp_i) = at(mid_i, 2) else {
        panic!()
    };
    let Wire::TractParams(tp_a) = at(mid_a, 2) else {
        panic!()
    };
    assert!(
        tp_i.valid && tp_a.valid,
        "inverse valid on both vowels: {tp_i:?} {tp_a:?}"
    );
    assert!(
        tp_a.q1 > tp_i.q1,
        "/ɑ/ is the high-q1 vowel: {tp_i:?} vs {tp_a:?}"
    );
    let Wire::AreaFunction(af_i) = at(mid_i, 3) else {
        panic!()
    };
    let Wire::AreaFunction(af_a) = at(mid_a, 3) else {
        panic!()
    };
    assert!(af_i.live && af_a.live);
    assert_ne!(af_i.diameters_cm, af_a.diameters_cm, "the tube moves");

    // Timing and deadline stats are populated, and judged against config.
    let budget = shell.stats.hop_budget_us.load(Ordering::Relaxed);
    assert_eq!(budget, (format.hop_seconds() as f64 * 1e6) as u64);
    for (i, s) in shell.stats.stages.iter().enumerate() {
        assert!(s.max_us.load(Ordering::Relaxed) > 0, "stage {i} timed");
    }
    assert!(shell.stats.worst_hop_us.load(Ordering::Relaxed) > 0);
    assert!(shell.stats.analysis_latency_max_us.load(Ordering::Relaxed) > 0);
    assert!(shell.config.hop_budget_fraction_max > 0.0);
    assert!(shell.config.mic_to_render_max_ms > 0.0);
    // The render leg is the shell's to record.
    shell.stats.note_render(msgs[0].captured_at);
    assert!(shell.stats.render_latency_last_us.load(Ordering::Relaxed) > 0);
}

#[test]
fn a_build_failure_is_reported_not_swallowed() {
    let mut def = live_model_coarse();
    def.stages.remove(1);
    let (rx_tx, rx) = rtrb::RingBuffer::<f32>::new(16);
    drop(rx_tx);
    let (mut runner, shell) = Runner::spawn(
        def.clone(),
        def.format(Some(SR)),
        rx,
        None,
        Arc::new(AtomicBool::new(false)),
    );
    assert_eq!(
        runner.wait_ready(Duration::from_secs(60), Duration::from_millis(5)),
        RunnerState::Failed
    );
    runner.stop();
    assert_eq!(shell.stats.state(), RunnerState::Failed);
    let text = super::runner::describe_build_failure(&def, def.format(Some(SR))).unwrap();
    assert!(text.contains("adapter chain"), "{text}");
}

/// D8's runner half: the wires are allocated at build and never grow.
#[test]
fn wires_are_preallocated_and_stable_across_hops() {
    let def = live_model_coarse();
    let mut p = build(&def, def.format(Some(SR))).unwrap();
    let cap_before = match &p.wires[0] {
        Wire::AudioFrame(f) => f.samples.capacity(),
        _ => unreachable!(),
    };
    let sig = vowel(120.0, 3.86, 1.35, 2048);
    if let Wire::AudioFrame(f) = &mut p.wires[0] {
        f.samples.copy_from_slice(&sig);
    }
    for _ in 0..3 {
        p.run_hop().unwrap();
    }
    let cap_after = match &p.wires[0] {
        Wire::AudioFrame(f) => f.samples.capacity(),
        _ => unreachable!(),
    };
    assert_eq!(cap_before, cap_after);
    assert_eq!(p.wires.len(), 5);
}
