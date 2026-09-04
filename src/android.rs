//! Android entry point.
//!
//! android-activity's NativeActivity glue (pulled in via winit's
//! `android-native-activity` backend) loads the cdylib in the Activity's
//! `onCreate` and then calls the `#[unsafe(no_mangle)] fn android_main` symbol
//! defined here on a dedicated thread. We wire up the *same* [`DashboardApp`]
//! egui app the desktop and web targets use.
//!
//! Two deliberate de-risking choices for this first on-device build:
//!   1. **CPU DSP only.** Analysis runs `math::yin_pitch` + LPC formants on the
//!      CPU (see [`cpu_analysis_loop`]); it never constructs the wgpu `GpuYin`
//!      compute path, so a missing/limited Vulkan driver can't break launch.
//!   2. **glow (GLES/EGL) renderer**, not wgpu/Vulkan — forced via
//!      `NativeOptions.renderer`.
//!
//! Microphone capture needs the `RECORD_AUDIO` runtime permission. It is
//! declared in the manifest, but Android only grants it once the user (or
//! `adb shell pm grant`) approves it. Until then cpal's input stream fails to
//! open; we treat that as non-fatal so the UI still launches (it just shows
//! "SEARCHING"). See docs/android-build.md.
//!
//! **Re-entry.** NativeActivity can call `android_main` again in the same
//! process — an activity relaunch under "Don't keep activities", a task
//! restart — while the previous call is still parked inside
//! `eframe::run_native`. winit 0.30.13 stubs `MainEvent::Destroy`, so the old
//! generation is never told to unwind and cannot clean up after itself. Rather
//! than chase a clean exit we make re-entry safe: [`retire_previous_generation`]
//! tears the last generation's audio streams and analysis thread down before
//! the new one builds its own, so relaunching cannot accumulate threads or
//! leave a second AAudio stream holding the microphone. Revisit when winit
//! ≥ 0.31 (with real `Destroy` handling) lands in eframe.

use crate::audio::AudioEngine;
use crate::concurrency::{AnalysisState, ConcurrencyBridges, Telemetry};
use crate::types::VocalProfile;
use crate::ui::DashboardApp;
use rtrb::Consumer;
use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use triple_buffer::Input;
use winit::platform::android::activity::AndroidApp;

use crate::frame::ANALYSIS_FRAME;

/// Fallback microphone rate when the input device can't be queried yet (e.g.
/// `RECORD_AUDIO` not granted at launch). 48 kHz is the near-universal Android
/// capture rate.
const FALLBACK_SAMPLE_RATE: f32 = 48_000.0;

/// Idle sleep between ring-buffer drains on the analysis thread. Also the
/// granularity at which it notices a shutdown request.
const ANALYSIS_POLL_MS: u64 = 5;

/// How long a new generation waits for the previous analysis thread to finish
/// before giving up and detaching it. The loop polls its shutdown flag every
/// [`ANALYSIS_POLL_MS`], so this is ~100x the expected wait: long enough that
/// it never fires in practice, short enough that a wedged thread can't stop
/// the activity from relaunching.
const SHUTDOWN_GRACE: Duration = Duration::from_millis(500);

/// Poll interval while waiting out [`SHUTDOWN_GRACE`].
const SHUTDOWN_POLL: Duration = Duration::from_millis(5);

/// Per-frame analysis times kept for the p95 estimate (~11 s at 48 kHz).
const TIMING_WINDOW: usize = 256;

/// How often, in frames, the frame-cost summary is logged. 512 frames is ~22 s
/// at 48 kHz — frequent enough to watch live over `adb logcat`, rare enough
/// not to be the noisiest thing in it.
const TIMING_REPORT_FRAMES: u32 = 512;

/// One `android_main` invocation's background resources, parked where the
/// *next* invocation can reach them.
struct Generation {
    /// Set to ask this generation's analysis loop to return.
    shutdown: Arc<AtomicBool>,
    analysis: Option<JoinHandle<()>>,
    /// Dropping this closes the AAudio input/output streams — the reason the
    /// engine lives here instead of on the parked `android_main` stack frame,
    /// which the new generation cannot reach.
    audio: Option<AudioEngine>,
}

/// The most recent generation, or `None` before the first launch. A poisoned
/// lock is recovered from rather than propagated: the data behind it is two
/// owned handles, and refusing to relaunch the app would be the worse failure.
static ACTIVE_GENERATION: Mutex<Option<Generation>> = Mutex::new(None);

/// Shuts down whatever the previous `android_main` left running. Safe to call
/// on a cold start, where there is nothing to retire.
fn retire_previous_generation() {
    let previous = match ACTIVE_GENERATION.lock() {
        Ok(mut slot) => slot.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    };
    let Some(mut previous) = previous else {
        return;
    };
    log::info!("retiring the previous android_main generation");

    // Ask the analysis loop to stop, then release the microphone immediately
    // rather than waiting on the thread — a second AAudio input stream opening
    // while the first still holds the mic is the failure this exists to stop.
    previous.shutdown.store(true, Ordering::Relaxed);
    drop(previous.audio.take());

    // The spatial (TV-path) capture thread holds its own AudioRecord; it
    // checks this flag every 0.1 s chunk and releases the recorder itself.
    crate::spatial::request_shutdown();

    if let Some(handle) = previous.analysis.take() {
        let deadline = Instant::now() + SHUTDOWN_GRACE;
        while !handle.is_finished() && Instant::now() < deadline {
            thread::sleep(SHUTDOWN_POLL);
        }
        if handle.is_finished() {
            // Never `join()` blind: a wedged loop would hang the relaunch.
            let _ = handle.join();
        } else {
            log::warn!(
                "previous analysis thread did not stop within {} ms; detaching it",
                SHUTDOWN_GRACE.as_millis()
            );
        }
    }
}

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    log::info!("android_main: starting Voice Harmonic Engine");

    // Before anything is constructed: whatever a previous launch left running
    // in this process must be gone. See the module docs on re-entry.
    retire_previous_generation();

    let bridges = ConcurrencyBridges::new();

    // Analysis thread inputs.
    let profile_tx = bridges.profile_tx;
    let ui_profile_tx = bridges.ui_profile_tx;
    let audio_rx = bridges.audio_rx;
    let spectrum_tx = bridges.spectrum_tx;
    let scope_tx = bridges.scope_tx;

    // Query the mic rate up front so DSP frequency scaling is correct; fall back
    // if unavailable (permission not yet granted / no input device).
    let input_sample_rate = query_input_sample_rate().unwrap_or(FALLBACK_SAMPLE_RATE);
    log::info!("android input sample rate: {input_sample_rate} Hz");

    // CPU YIN + LPC on the non-real-time analysis thread. A panic in here used
    // to unwind into nothing and leave the UI showing its last profile forever;
    // now the death is caught, logged to logcat, and published as
    // `AnalysisState::Stopped` so the UI can say analysis has stopped.
    let analysis_telemetry = bridges.telemetry.clone();
    let shutdown = Arc::new(AtomicBool::new(false));
    let loop_shutdown = shutdown.clone();
    let analysis = thread::spawn(move || {
        analysis_telemetry.set_analysis_state(AnalysisState::Running);
        let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| {
            cpu_analysis_loop(
                profile_tx,
                ui_profile_tx,
                audio_rx,
                spectrum_tx,
                scope_tx,
                input_sample_rate,
                &analysis_telemetry,
                &loop_shutdown,
            );
        }));
        // A requested shutdown is a relaunch, not a fault: it must not be
        // logged as an error, and the retired generation's telemetry must not
        // be marked `Stopped` (nothing reads it, but the log would mislead).
        if loop_shutdown.load(Ordering::Relaxed) {
            log::info!("analysis loop stopped for an activity relaunch");
            return;
        }
        match outcome {
            Ok(()) => log::error!("analysis loop exited; live analysis has stopped"),
            Err(_) => log::error!("analysis loop panicked; live analysis has stopped"),
        }
        analysis_telemetry.set_analysis_state(AnalysisState::Stopped);
    });

    // cpal AAudio engine. Non-fatal on failure: without RECORD_AUDIO the input
    // stream can't open, but we still want the UI up so the user can grant it.
    let audio = match AudioEngine::start(
        bridges.profile_rx,
        bridges.event_rx,
        bridges.audio_tx,
        bridges.telemetry.clone(),
    ) {
        Ok(engine) => Some(engine),
        Err(e) => {
            log::error!(
                "audio engine failed to start (is RECORD_AUDIO granted?): {e:?}; UI will run without audio"
            );
            // The UI must say this. A NativeActivity cannot raise the runtime
            // permission dialog itself, so without a banner the app looks
            // merely broken rather than un-permitted, and the only diagnosis
            // is a logcat line most users will never read.
            bridges.telemetry.set_audio_unavailable(true);
            None
        }
    };

    // Park both handles where the *next* `android_main` can find them. This
    // invocation may never get the chance to clean up after itself.
    let generation = Generation {
        shutdown,
        analysis: Some(analysis),
        audio,
    };
    match ACTIVE_GENERATION.lock() {
        Ok(mut slot) => *slot = Some(generation),
        Err(poisoned) => *poisoned.into_inner() = Some(generation),
    }

    let event_tx = bridges.event_tx;
    let telemetry = bridges.telemetry.clone();
    let ui_profile_rx = bridges.ui_profile_rx;
    let spectrum_rx = bridges.spectrum_rx;
    let scope_rx = bridges.scope_rx;

    // The reference + archive persist here — the app's private internal storage
    // (`/data/data/<pkg>/files`), which is writable without any permission and
    // cleared only on uninstall. Captured before `app` moves into the options.
    let store_path = app.internal_data_path().map(|p| p.join("archive.json"));
    // Raw capture WAVs go to app-specific *external* storage
    // (`/sdcard/Android/data/<pkg>/files/captures`): no permission needed,
    // and unlike internal storage it is reachable with a plain `adb pull`
    // (no `run-as`), which is how the study harness collects them.
    let external = app
        .external_data_path()
        .or_else(|| app.internal_data_path());
    let paths = crate::ui::AppPaths {
        store: store_path,
        captures: external.as_ref().map(|p| p.join("captures")),
        // The import folder: `adb push`, a file manager, or the share
        // sheet (see `share_intent`) puts audio here; the Sessions screen
        // lists and analyzes it.
        imports: external.as_ref().map(|p| p.join("import")),
    };
    // A file shared to the app ("Share → VoxLabs") arrives as this
    // activity's intent; copy it into the import folder now and hand it to
    // the UI to analyze on its first frame.
    let shared = paths
        .imports
        .as_deref()
        .and_then(|dir| crate::share_intent::take_shared_audio(dir));

    let native_options = eframe::NativeOptions {
        android_app: Some(app),
        // glow/EGL (GLES) rather than wgpu/Vulkan for the first build.
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "Voice Harmonic Engine",
        native_options,
        Box::new(move |cc| {
            let mut app = DashboardApp::new(
                cc,
                event_tx,
                telemetry,
                ui_profile_rx,
                spectrum_rx,
                scope_rx,
                input_sample_rate,
                paths,
            );
            if let Some(path) = shared {
                app.queue_import(path);
            }
            Ok(Box::new(app))
        }),
    ) {
        log::error!("eframe exited with error: {e:?}");
    }
}

/// Best-effort read of the default input device's sample rate via cpal.
fn query_input_sample_rate() -> Option<f32> {
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    let dev = host.default_input_device()?;
    let config = dev.default_input_config().ok()?;
    Some(config.config().sample_rate as f32)
}

/// CPU-only analysis loop: drains the input ring buffer, and for each whole
/// frame estimates f0 (YIN) and — on voiced frames — the formants (decimate →
/// LPC → root-solve), publishing the profile to synthesis + UI. This mirrors
/// `AnalysisEngine::process_frame`'s CPU path but never touches wgpu.
#[allow(clippy::too_many_arguments)]
fn cpu_analysis_loop(
    mut profile_tx: Input<VocalProfile>,
    mut ui_profile_tx: Input<VocalProfile>,
    mut audio_rx: Consumer<f32>,
    mut spectrum_tx: Input<Vec<f32>>,
    mut scope_tx: Input<Vec<f32>>,
    sample_rate: f32,
    telemetry: &Telemetry,
    shutdown: &AtomicBool,
) {
    use crate::frame::FrameAnalyzer;
    use crate::math;

    // The per-frame DSP is the shared `FrameAnalyzer` (identical to what the
    // `voxlab` study harness runs on files); this loop owns only the ring
    // drain, room-calibration bookkeeping, UI feeds, and timing.
    let mut analyzer = FrameAnalyzer::new(sample_rate);
    let mut accumulator: Vec<f32> = Vec::with_capacity(ANALYSIS_FRAME * 4);
    // Scrolling-spectrogram STFT, mirroring the desktop path.
    let mut spectrogram = crate::spectrogram::Spectrogram::new();
    // Room calibration state, mirroring the desktop engine.
    let mut calibrator = math::RoomCalibrator::new();
    let mut timer = FrameTimer::new(sample_rate);

    loop {
        if shutdown.load(Ordering::Relaxed) {
            return;
        }

        while let Ok(sample) = audio_rx.pop() {
            accumulator.push(sample);
        }

        while accumulator.len() >= ANALYSIS_FRAME {
            // A large backlog can hold us in here for many frames, so the
            // shutdown request is checked at frame granularity too.
            if shutdown.load(Ordering::Relaxed) {
                return;
            }
            let started = Instant::now();
            let frame = &accumulator[..ANALYSIS_FRAME];

            // Mirror the frame to the capture export (no-op unless a capture
            // is armed) before anything else sees it.
            crate::capture_log::push(frame);
            let result = analyzer.analyze(frame);

            // Room calibration pass, mirroring the desktop engine. It wants
            // the pre-gate periodicity (a calibrating room with a voice in
            // it must fail), which the analyzer reports alongside the
            // gated profile.
            let (calibrating, calib_done) = telemetry.take_calibration_frame();
            if calibrating {
                calibrator.push(result.rms, result.yin_f0);
                if calib_done {
                    match calibrator.finish() {
                        Ok(cal) => {
                            analyzer.seed_floor(cal.ambient_rms);
                            analyzer.set_interferer(cal.interferer);
                            telemetry.set_calibration_result(Some((
                                cal.ambient_rms,
                                cal.interferer.map(|i| (i.f0_hz, i.rms)),
                            )));
                            log::info!(
                                "room calibrated: ambient rms {:.5}, interferer {:?}",
                                cal.ambient_rms,
                                cal.interferer
                            );
                        }
                        Err(e) => {
                            log::warn!("room calibration failed: {e:?}");
                            telemetry.set_calibration_result(None);
                        }
                    }
                    calibrator = math::RoomCalibrator::new();
                }
            }

            let profile = result.profile;
            profile_tx.write(profile);
            ui_profile_tx.write(profile);

            // Spectrogram + oscilloscope feeds, mirroring the desktop path.
            spectrogram.process_block(frame);
            spectrum_tx.write(spectrogram.magnitudes_db().to_vec());
            scope_tx.write(frame.to_vec());

            // Heartbeat for the UI's staleness watch.
            telemetry.note_analysis_frame();

            accumulator.drain(..ANALYSIS_FRAME);
            timer.record(started.elapsed());
        }

        thread::sleep(Duration::from_millis(ANALYSIS_POLL_MS));
    }
}

/// Rolling per-frame cost tracker for the Android analysis loop.
///
/// One analysis frame covers `ANALYSIS_FRAME / sample_rate` seconds of audio
/// (42.7 ms at 48 kHz). Take longer than that on average and the input ring
/// buffer backs up until the capture callback starts overrunning, which shows
/// up as xruns and dropped input rather than as an obvious slowdown — so the
/// number is worth logging even when it is comfortably inside budget.
///
/// Deliberately a measurement, not a control loop: the review's F25 asks for
/// the benchmark first and an adaptive cadence *only* if a real device shows
/// the budget being exceeded.
struct FrameTimer {
    budget_ms: f32,
    recent: Vec<f32>,
    next: usize,
    since_report: u32,
}

impl FrameTimer {
    fn new(sample_rate: f32) -> Self {
        Self {
            budget_ms: ANALYSIS_FRAME as f32 / sample_rate * 1000.0,
            recent: Vec::with_capacity(TIMING_WINDOW),
            next: 0,
            since_report: 0,
        }
    }

    fn record(&mut self, elapsed: Duration) {
        let ms = elapsed.as_secs_f32() * 1000.0;
        if self.recent.len() < TIMING_WINDOW {
            self.recent.push(ms);
        } else {
            self.recent[self.next] = ms;
            self.next = (self.next + 1) % TIMING_WINDOW;
        }

        self.since_report += 1;
        if self.since_report < TIMING_REPORT_FRAMES {
            return;
        }
        self.since_report = 0;
        self.report();
    }

    fn report(&self) {
        if self.recent.is_empty() {
            return;
        }
        let mut sorted = self.recent.clone();
        sorted.sort_by(f32::total_cmp);
        let at = |q: f32| sorted[((sorted.len() - 1) as f32 * q).round() as usize];
        let (p50, p95, max) = (at(0.5), at(0.95), sorted[sorted.len() - 1]);
        let headroom = p95 / self.budget_ms * 100.0;

        if p95 > self.budget_ms {
            log::warn!(
                "analysis frame cost over budget: p50 {p50:.1} ms, p95 {p95:.1} ms, \
                 max {max:.1} ms vs {:.1} ms budget ({headroom:.0}%)",
                self.budget_ms
            );
        } else {
            log::info!(
                "analysis frame cost: p50 {p50:.1} ms, p95 {p95:.1} ms, max {max:.1} ms \
                 vs {:.1} ms budget ({headroom:.0}%)",
                self.budget_ms
            );
        }
    }
}
