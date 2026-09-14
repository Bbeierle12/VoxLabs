//! The live engine: one runner on the live ring buffer, started by mode
//! name, switchable at runtime, retired on relaunch. The UI requests a
//! mode (`request_mode`); a watcher thread performs the switch on the
//! engine (stop the runner, take back the ring-buffer consumer and the
//! observer, spawn the next mode) and parks the new `ShellHandle` where
//! the UI picks it up (`take_pending_handle`). The chosen mode persists
//! in `mode.txt` beside the archive so a relaunch comes back the same.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use rtrb::Consumer;

use crate::concurrency::AnalysisState;
use crate::pipeline::PipelineDefinition;
use crate::pipeline::runner::{HopObserver, Parked, Runner, RunnerState, RunnerStats, ShellHandle};

/// How often the watcher looks for a mode request.
const WATCH_POLL: Duration = Duration::from_millis(200);
/// The mode a fresh install runs.
pub const DEFAULT_MODE: &str = "live_model";
/// The file the chosen mode persists in.
pub const MODE_FILE: &str = "mode.txt";

static REQUEST: Mutex<Option<String>> = Mutex::new(None);
static PENDING: Mutex<Option<ShellHandle>> = Mutex::new(None);
static CURRENT: Mutex<String> = Mutex::new(String::new());
static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);
static SWITCHING: AtomicBool = AtomicBool::new(false);

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// The compiled-in mode names, in the order the UI offers them.
pub fn mode_names() -> Vec<&'static str> {
    PipelineDefinition::MODES.iter().map(|(n, _)| *n).collect()
}

/// Asks the watcher to switch the live runner to `name`.
pub fn request_mode(name: &str) {
    *lock(&REQUEST) = Some(name.to_string());
}

pub fn take_pending_handle() -> Option<ShellHandle> {
    lock(&PENDING).take()
}

pub fn current_mode() -> String {
    lock(&CURRENT).clone()
}

pub fn is_switching() -> bool {
    SWITCHING.load(Ordering::Relaxed)
}

pub fn last_error() -> Option<String> {
    lock(&LAST_ERROR).clone()
}

/// The persisted mode, or the default; an unknown name is ignored.
pub fn saved_mode(dir: Option<&Path>) -> String {
    let saved = dir
        .and_then(|d| std::fs::read_to_string(d.join(MODE_FILE)).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| mode_names().contains(&s.as_str()));
    saved.unwrap_or_else(|| DEFAULT_MODE.to_string())
}

pub fn save_mode(dir: Option<&Path>, name: &str) {
    if let Some(d) = dir
        && let Err(e) = std::fs::write(d.join(MODE_FILE), name)
    {
        log::warn!("could not persist the mode: {e}");
    }
}

pub struct LiveEngine {
    sample_rate: f32,
    runner: Option<Runner>,
    runner_shutdown: Option<Arc<AtomicBool>>,
    parked: Option<Parked>,
    telemetry: Arc<crate::concurrency::Telemetry>,
}

impl LiveEngine {
    pub fn new(
        sample_rate: f32,
        audio_rx: Consumer<f32>,
        observer: Option<Box<dyn HopObserver>>,
        telemetry: Arc<crate::concurrency::Telemetry>,
    ) -> Self {
        Self {
            sample_rate,
            runner: None,
            runner_shutdown: None,
            parked: Some((audio_rx, observer)),
            telemetry,
        }
    }

    /// Starts `mode` on the parked audio stream. Waits for the worker's
    /// build (bounded by the mode's `init_timeout_ms`) and reports the
    /// analysis state to telemetry.
    pub fn start(&mut self, mode: &str) -> Result<ShellHandle, String> {
        let def = PipelineDefinition::by_name_or_path(mode).map_err(|e| format!("{mode}: {e}"))?;
        let (audio_rx, mut observer) = self
            .parked
            .take()
            .ok_or("the audio stream is not parked (the previous runner did not hand it back)")?;
        let format = def.format(Some(self.sample_rate));
        if let Some(obs) = observer.as_mut() {
            obs.set_cadence(def.runner.legacy_frame_every_hops, format.hop);
        }
        let init_timeout = Duration::from_millis(def.runner.init_timeout_ms);
        let poll = Duration::from_millis(def.runner.poll_ms);
        let shutdown = Arc::new(AtomicBool::new(false));
        let (runner, handle) = Runner::spawn(def, format, audio_rx, observer, shutdown.clone());
        let state = runner.wait_ready(init_timeout, poll);
        self.runner = Some(runner);
        self.runner_shutdown = Some(shutdown);
        match state {
            RunnerState::Running => self.telemetry.set_analysis_state(AnalysisState::Running),
            RunnerState::Building => {
                log::warn!(
                    "pipeline `{mode}` still building after {} ms; continuing",
                    init_timeout.as_millis()
                );
                self.telemetry.set_analysis_state(AnalysisState::Running);
            }
            RunnerState::Failed | RunnerState::Stopped => {
                self.telemetry
                    .set_analysis_state(AnalysisState::Unavailable);
                // Take the stream back so the next mode can try.
                self.park();
                return Err(format!("pipeline `{mode}` failed to start (see log)"));
            }
        }
        *lock(&CURRENT) = mode.to_string();
        Ok(handle)
    }

    /// Stops the runner and parks what it hands back.
    fn park(&mut self) {
        if let Some(mut r) = self.runner.take()
            && let Some(p) = r.stop()
        {
            self.parked = Some(p);
        }
        self.runner_shutdown = None;
    }

    /// Stops the current mode and starts `mode` on the same stream.
    pub fn switch(&mut self, mode: &str) -> Result<ShellHandle, String> {
        self.park();
        self.start(mode)
    }

    pub fn stats(&self) -> Option<Arc<RunnerStats>> {
        self.runner.as_ref().map(|r| r.stats())
    }

    /// Relaunch path: ask the worker to stop, wait up to `grace`, join if
    /// it stopped, detach it otherwise (never block on a wedged thread).
    pub fn retire(&mut self, grace: Duration, poll: Duration) {
        if let Some(flag) = &self.runner_shutdown {
            flag.store(true, Ordering::Relaxed);
        }
        if let Some(mut runner) = self.runner.take() {
            let stats = runner.stats();
            let deadline = Instant::now() + grace;
            let done = |s: RunnerState| matches!(s, RunnerState::Stopped | RunnerState::Failed);
            while !done(stats.state()) && Instant::now() < deadline {
                thread::sleep(poll);
            }
            if done(stats.state()) {
                let _ = runner.stop();
            } else {
                log::warn!(
                    "pipeline runner did not stop within {} ms; detaching it",
                    grace.as_millis()
                );
                runner.detach();
            }
        }
    }
}

/// Runs mode requests against `engine` until `shutdown`.
pub fn spawn_watcher(
    engine: Arc<Mutex<LiveEngine>>,
    shutdown: Arc<AtomicBool>,
    mode_dir: Option<PathBuf>,
) {
    thread::Builder::new()
        .name("vox-mode-watcher".into())
        .spawn(move || {
            while !shutdown.load(Ordering::Relaxed) {
                thread::sleep(WATCH_POLL);
                let Some(name) = lock(&REQUEST).take() else {
                    continue;
                };
                if name == current_mode() {
                    continue;
                }
                SWITCHING.store(true, Ordering::Relaxed);
                let outcome = lock(&engine).switch(&name);
                match outcome {
                    Ok(handle) => {
                        *lock(&PENDING) = Some(handle);
                        *lock(&LAST_ERROR) = None;
                        save_mode(mode_dir.as_deref(), &name);
                        log::info!("live pipeline switched to `{name}`");
                    }
                    Err(e) => {
                        log::error!("mode switch to `{name}` failed: {e}");
                        *lock(&LAST_ERROR) = Some(e);
                    }
                }
                SWITCHING.store(false, Ordering::Relaxed);
            }
        })
        .expect("spawn mode watcher");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A runner can be stopped and another mode started on the same
    /// stream: the consumer and observer come back from the worker.
    #[test]
    fn switching_modes_reuses_the_parked_stream() {
        let telemetry = Arc::new(crate::concurrency::Telemetry::new());
        let (mut tx, rx) = rtrb::RingBuffer::<f32>::new(1 << 16);
        for _ in 0..4096 {
            let _ = tx.push(0.0);
        }
        let mut engine = LiveEngine::new(48_000.0, rx, None, telemetry);
        let a = engine.start("calibrate").unwrap();
        assert_eq!(a.pipeline_name, "calibrate");
        assert_eq!(current_mode(), "calibrate");
        let b = engine.switch("choir").unwrap();
        assert_eq!(b.pipeline_name, "choir");
        assert_eq!(b.format.frame_samples, 8192);
        assert!(engine.stats().is_some());
        assert!(engine.start("nope").is_err() || engine.parked.is_none());
        engine.retire(Duration::from_millis(500), Duration::from_millis(5));
        assert!(engine.runner.is_none());
        let d = std::env::temp_dir();
        save_mode(Some(&d), "choir");
        assert_eq!(saved_mode(Some(&d)), "choir");
        save_mode(Some(&d), "not-a-mode");
        assert_eq!(saved_mode(Some(&d)), DEFAULT_MODE);
    }
}
