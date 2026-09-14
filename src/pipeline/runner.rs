//! The runner: one worker thread per pipeline. It drains the audio ring
//! buffer, re-frames the samples to the hop itself (D6: Android may ignore
//! `BufferSize::Fixed`, cpal #902, so the callback's chunking is never
//! trusted), runs the stages once per hop with per-stage timing and
//! deadline accounting, publishes the taps, and gives the shell's
//! not-yet-wrapped per-frame work a hook at its old cadence.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use rtrb::Consumer;

use crate::config::consts::MICROS_PER_SECOND_F64;

use super::builder::{Pipeline, WiringError, build};
use super::definition::{PipelineDefinition, RunnerConfig};
use super::stage::StreamFormat;
use super::tap::{TapMsg, TapReceiver, TapSender, tap_channel};
use super::types::{AudioFrame, Wire};

/// Per-stage wall-time accounting, microseconds.
#[derive(Default)]
pub struct StageStats {
    pub last_us: AtomicU64,
    pub max_us: AtomicU64,
    pub total_us: AtomicU64,
}

/// Worker lifecycle as the shell sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RunnerState {
    Building,
    Running,
    /// The pipeline could not be built; the error is in the log.
    Failed,
    Stopped,
}

impl RunnerState {
    /// Decode order = declaration order (the `repr(u8)` discriminants).
    const ALL: &'static [RunnerState] = &[
        RunnerState::Building,
        RunnerState::Running,
        RunnerState::Failed,
        RunnerState::Stopped,
    ];
}

/// Everything the shell reads: counters written by the worker (and, for
/// the render leg of the latency, by the shell itself at paint time).
pub struct RunnerStats {
    state: AtomicU8,
    pub hops: AtomicU64,
    /// Hops whose processing exceeded the full hop budget.
    pub misses: AtomicU64,
    /// Hops whose processing exceeded the gate fraction of the budget.
    pub over_fraction: AtomicU64,
    pub stage_errors: AtomicU64,
    pub last_hop_us: AtomicU64,
    pub worst_hop_us: AtomicU64,
    pub hop_budget_us: AtomicU64,
    /// Ring arrival → taps published (the analysis leg), microseconds.
    pub analysis_latency_last_us: AtomicU64,
    pub analysis_latency_max_us: AtomicU64,
    /// Ring arrival → tap consumed at paint (mic-to-render, minus the
    /// device's own capture latency), written by the shell.
    pub render_latency_last_us: AtomicU64,
    pub render_latency_max_us: AtomicU64,
    /// Samples waiting in the ring beyond one hop when a hop started.
    pub backlog_max_samples: AtomicU64,
    pub tap_drops: Arc<AtomicU64>,
    pub stages: Vec<StageStats>,
    pub stage_names: Vec<Arc<str>>,
}

impl RunnerStats {
    pub fn state(&self) -> RunnerState {
        let raw = self.state.load(Ordering::Relaxed) as usize;
        RunnerState::ALL
            .get(raw)
            .copied()
            .unwrap_or(RunnerState::Building)
    }

    fn set_state(&self, s: RunnerState) {
        self.state.store(s as u8, Ordering::Relaxed);
    }

    /// Shell side: record one tap's mic-to-render leg at paint time.
    pub fn note_render(&self, captured_at: Instant) {
        let us = captured_at.elapsed().as_micros() as u64;
        self.render_latency_last_us.store(us, Ordering::Relaxed);
        self.render_latency_max_us.fetch_max(us, Ordering::Relaxed);
    }
}

fn max_store(slot: &AtomicU64, v: u64) {
    slot.fetch_max(v, Ordering::Relaxed);
}

/// The shell's per-hop hook for work the pipeline does not wrap yet.
pub trait HopObserver: Send {
    /// `frame` is the hop's full analysis frame; `wires` every stage output
    /// (wire 0 is the frame). Called after the stages, before the taps.
    fn on_hop(&mut self, frame: &AudioFrame, wires: &[Wire], hop: u64);
}

/// What the shell holds: taps, stats, and the thresholds to judge them by.
pub struct ShellHandle {
    pub taps: TapReceiver,
    pub stats: Arc<RunnerStats>,
    pub config: RunnerConfig,
    pub format: StreamFormat,
    pub pipeline_name: String,
    /// (name, backend) per stage, in order — for the shell's panel.
    pub stages: Vec<(String, String)>,
}

/// The worker's owner. Dropping it asks the worker to stop and joins it.
pub struct Runner {
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    stats: Arc<RunnerStats>,
}

impl Runner {
    /// Spawns the worker. The pipeline is built on the worker (its `init`
    /// may take a moment — the grid builds), so the shell should
    /// [`Runner::wait_ready`] before opening the audio device.
    pub fn spawn(
        def: PipelineDefinition,
        format: StreamFormat,
        audio_rx: Consumer<f32>,
        observer: Option<Box<dyn HopObserver>>,
        shutdown: Arc<AtomicBool>,
    ) -> (Runner, ShellHandle) {
        let cfg = def.runner;
        let (tap_tx, tap_rx) = tap_channel(cfg.tap_capacity);
        let stats = Arc::new(RunnerStats {
            state: AtomicU8::new(RunnerState::Building as u8),
            hops: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            over_fraction: AtomicU64::new(0),
            stage_errors: AtomicU64::new(0),
            last_hop_us: AtomicU64::new(0),
            worst_hop_us: AtomicU64::new(0),
            hop_budget_us: AtomicU64::new(
                (format.hop_seconds() as f64 * MICROS_PER_SECOND_F64) as u64,
            ),
            analysis_latency_last_us: AtomicU64::new(0),
            analysis_latency_max_us: AtomicU64::new(0),
            render_latency_last_us: AtomicU64::new(0),
            render_latency_max_us: AtomicU64::new(0),
            backlog_max_samples: AtomicU64::new(0),
            tap_drops: tap_tx.drops(),
            stages: def.stages.iter().map(|_| StageStats::default()).collect(),
            stage_names: def
                .stages
                .iter()
                .map(|s| Arc::from(s.name.as_str()))
                .collect(),
        });
        let name = def.name.clone();
        let stage_list: Vec<(String, String)> = def
            .stages
            .iter()
            .map(|s| (s.name.clone(), s.backend.clone()))
            .collect();
        let worker_stats = stats.clone();
        let worker_shutdown = shutdown.clone();
        let handle = thread::Builder::new()
            .name(format!("vox-runner-{name}"))
            .spawn(move || {
                // A panic in a stage or observer must not vanish with the
                // thread: it is logged and the state says the worker died.
                let stats_for_panic = worker_stats.clone();
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    worker(
                        def,
                        format,
                        audio_rx,
                        observer,
                        tap_tx,
                        worker_stats,
                        worker_shutdown,
                    )
                }));
                if outcome.is_err() {
                    log::error!("pipeline worker panicked; live analysis has stopped");
                    stats_for_panic.set_state(RunnerState::Failed);
                }
            })
            .expect("spawn runner thread");
        (
            Runner {
                shutdown,
                handle: Some(handle),
                stats: stats.clone(),
            },
            ShellHandle {
                taps: tap_rx,
                stats,
                config: cfg,
                format,
                pipeline_name: name,
                stages: stage_list,
            },
        )
    }

    /// Waits until the worker has built its pipeline (or failed), up to
    /// `timeout`. Returns the state it saw last.
    pub fn wait_ready(&self, timeout: Duration, poll: Duration) -> RunnerState {
        let deadline = Instant::now() + timeout;
        loop {
            let s = self.stats.state();
            if s != RunnerState::Building || Instant::now() >= deadline {
                return s;
            }
            thread::sleep(poll);
        }
    }

    pub fn stats(&self) -> Arc<RunnerStats> {
        self.stats.clone()
    }

    pub fn stop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }

    /// Asks the worker to stop without joining it — for a shell that must
    /// not block on a possibly wedged thread (the Android relaunch path).
    pub fn detach(mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        self.handle.take();
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Re-frames a sample stream to fixed hops: the first hop needs a whole
/// frame, every later hop `hop` new samples shifted into the frame.
struct Framer {
    frame: Vec<f32>,
    pending: Vec<f32>,
    hop: usize,
    primed: bool,
}

impl Framer {
    fn new(format: &StreamFormat, pending_frames: usize) -> Self {
        Self {
            frame: vec![0.0; format.frame_samples],
            pending: Vec::with_capacity(format.frame_samples * pending_frames),
            hop: format.hop,
            primed: false,
        }
    }

    fn need(&self) -> usize {
        if self.primed {
            self.hop
        } else {
            self.frame.len()
        }
    }

    /// Advances one hop if enough samples are pending; the frame then holds
    /// the newest `frame_samples` samples.
    fn advance(&mut self) -> bool {
        let need = self.need();
        if self.pending.len() < need {
            return false;
        }
        let len = self.frame.len();
        if self.primed {
            self.frame.copy_within(self.hop..len, 0);
            self.frame[len - self.hop..].copy_from_slice(&self.pending[..need]);
        } else {
            self.frame.copy_from_slice(&self.pending[..need]);
            self.primed = true;
        }
        self.pending.drain(..need);
        true
    }
}

#[allow(clippy::too_many_arguments)]
fn worker(
    def: PipelineDefinition,
    format: StreamFormat,
    mut audio_rx: Consumer<f32>,
    mut observer: Option<Box<dyn HopObserver>>,
    taps: TapSender,
    stats: Arc<RunnerStats>,
    shutdown: Arc<AtomicBool>,
) {
    let cfg = def.runner;
    let mut pipeline: Pipeline = match build(&def, format) {
        Ok(p) => p,
        Err(e) => {
            log::error!("pipeline `{}` failed to build:\n{e}", def.name);
            stats.set_state(RunnerState::Failed);
            return;
        }
    };
    let names: Vec<Arc<str>> = stats.stage_names.clone();
    let budget = Duration::from_secs_f32(format.hop_seconds());
    let budget_us = budget.as_micros() as u64;
    let fraction_us =
        (budget.as_secs_f64() * cfg.hop_budget_fraction_max as f64 * MICROS_PER_SECOND_F64) as u64;
    let mut timings = vec![0.0f64; pipeline.stages.len()];
    let mut framer = Framer::new(&format, cfg.pending_capacity_frames);
    let mut hop: u64 = 0;
    let poll = Duration::from_millis(cfg.poll_ms);
    log::info!(
        "pipeline `{}` running: {} stages, hop {} @ {:.0} Hz ({} us budget, gate {} us)",
        def.name,
        pipeline.stages.len(),
        format.hop,
        format.sample_rate_hz,
        budget_us,
        fraction_us
    );
    stats.set_state(RunnerState::Running);

    while !shutdown.load(Ordering::Relaxed) {
        while let Ok(s) = audio_rx.pop() {
            framer.pending.push(s);
        }
        let drained_at = Instant::now();
        max_store(
            &stats.backlog_max_samples,
            framer.pending.len().saturating_sub(framer.need()) as u64,
        );

        while framer.advance() {
            if shutdown.load(Ordering::Relaxed) {
                break;
            }
            let started = Instant::now();
            // Source wire: the newest frame.
            if let Wire::AudioFrame(src) = &mut pipeline.wires[0] {
                src.samples.copy_from_slice(&framer.frame);
                src.frame_index = hop;
                src.sample_rate = format.sample_rate_hz;
            }
            let result = pipeline.run_hop_timed(&mut timings);
            for (i, t) in timings.iter().enumerate() {
                let us = (*t * MICROS_PER_SECOND_F64) as u64;
                stats.stages[i].last_us.store(us, Ordering::Relaxed);
                max_store(&stats.stages[i].max_us, us);
                stats.stages[i].total_us.fetch_add(us, Ordering::Relaxed);
            }
            match result {
                Ok(()) => {
                    if let Some(obs) = observer.as_mut()
                        && let Wire::AudioFrame(src) = &pipeline.wires[0]
                    {
                        obs.on_hop(src, &pipeline.wires, hop);
                    }
                    let published_at = Instant::now();
                    for &stage_idx in &pipeline.taps {
                        let out = pipeline.stages[stage_idx].out;
                        taps.publish(TapMsg {
                            stage: stage_idx,
                            stage_name: names[stage_idx].clone(),
                            hop,
                            captured_at: drained_at,
                            published_at,
                            value: pipeline.wires[out].clone(),
                        });
                    }
                    let lat = (published_at - drained_at).as_micros() as u64;
                    stats.analysis_latency_last_us.store(lat, Ordering::Relaxed);
                    max_store(&stats.analysis_latency_max_us, lat);
                }
                Err((i, e)) => {
                    stats.stage_errors.fetch_add(1, Ordering::Relaxed);
                    log::error!(
                        "pipeline `{}` hop {hop}: stage `{}` failed: {e}; downstream stages skipped",
                        def.name,
                        names[i]
                    );
                }
            }
            let elapsed_us = started.elapsed().as_micros() as u64;
            stats.last_hop_us.store(elapsed_us, Ordering::Relaxed);
            max_store(&stats.worst_hop_us, elapsed_us);
            if elapsed_us > budget_us {
                stats.misses.fetch_add(1, Ordering::Relaxed);
            }
            if elapsed_us > fraction_us {
                stats.over_fraction.fetch_add(1, Ordering::Relaxed);
            }
            hop += 1;
            stats.hops.store(hop, Ordering::Relaxed);
            if hop % cfg.report_every_hops == 0 {
                report(&def.name, &stats, &names, hop, budget_us);
            }
        }
        thread::sleep(poll);
    }
    stats.set_state(RunnerState::Stopped);
}

/// Periodic timing line for logcat / the desktop log — the record a
/// ten-minute on-device run leaves behind.
fn report(name: &str, stats: &RunnerStats, names: &[Arc<str>], hop: u64, budget_us: u64) {
    let mut evidence = std::collections::BTreeMap::new();
    let per_stage: Vec<String> = stats
        .stages
        .iter()
        .zip(names)
        .map(|(s, n)| {
            let mean = s.total_us.load(Ordering::Relaxed) / hop.max(1);
            let max = s.max_us.load(Ordering::Relaxed);
            evidence.insert(format!("stage.{n}.mean_us"), mean.to_string());
            evidence.insert(format!("stage.{n}.max_us"), max.to_string());
            format!("{n} mean {mean} us max {max} us")
        })
        .collect();
    let load = |a: &AtomicU64| a.load(Ordering::Relaxed);
    for (k, v) in [
        ("hop", hop),
        ("worst_hop_us", load(&stats.worst_hop_us)),
        ("budget_us", budget_us),
        ("misses", load(&stats.misses)),
        ("over_gate", load(&stats.over_fraction)),
        (
            "analysis_latency_max_us",
            load(&stats.analysis_latency_max_us),
        ),
        ("render_latency_max_us", load(&stats.render_latency_max_us)),
        ("backlog_max_samples", load(&stats.backlog_max_samples)),
        ("tap_drops", load(&stats.tap_drops)),
        ("stage_errors", load(&stats.stage_errors)),
    ] {
        evidence.insert(k.to_string(), v.to_string());
    }
    // The same line as an event, so a phone without adb keeps the record
    // in its diagnostics bundle (Phase 1 gate, D18).
    let severity = if load(&stats.misses) > 0 || load(&stats.stage_errors) > 0 {
        crate::diagnostics::Severity::Warning
    } else {
        crate::diagnostics::Severity::Info
    };
    crate::diagnostics::runtime::log(
        "runtime",
        "pipeline_report",
        &format!(
            "pipeline `{name}` hop {hop}: worst hop {} us of {budget_us} us budget, misses {}, \
             stage errors {}",
            load(&stats.worst_hop_us),
            load(&stats.misses),
            load(&stats.stage_errors)
        ),
        evidence,
        severity,
    );
    log::info!(
        "pipeline `{name}` hop {hop}: worst hop {} us of {budget_us} us budget, misses {}, over-gate {}, \
         analysis latency max {} us, render latency max {} us, backlog max {} samples, tap drops {}, \
         stage errors {}; {}",
        stats.worst_hop_us.load(Ordering::Relaxed),
        stats.misses.load(Ordering::Relaxed),
        stats.over_fraction.load(Ordering::Relaxed),
        stats.analysis_latency_max_us.load(Ordering::Relaxed),
        stats.render_latency_max_us.load(Ordering::Relaxed),
        stats.backlog_max_samples.load(Ordering::Relaxed),
        stats.tap_drops.load(Ordering::Relaxed),
        stats.stage_errors.load(Ordering::Relaxed),
        per_stage.join("; ")
    );
}

/// A wiring failure surfaced to the shell after `spawn`: the worker logs
/// the full adapter chain; this is the same text for a banner.
pub fn describe_build_failure(def: &PipelineDefinition, format: StreamFormat) -> Option<String> {
    match build(def, format) {
        Ok(_) => None,
        Err(e) => Some(format!("{e}")),
    }
}

#[allow(dead_code)]
fn _assert_error_display(e: WiringError) -> String {
    e.to_string()
}
