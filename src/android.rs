//! Android entry point.
//!
//! android-activity's NativeActivity glue (pulled in via winit's
//! `android-native-activity` backend) loads the cdylib in the Activity's
//! `onCreate` and then calls the `#[unsafe(no_mangle)] fn android_main` symbol
//! defined here on a dedicated thread. We wire up the *same* [`DashboardApp`]
//! egui app the desktop and web targets use.
//!
//! Two deliberate de-risking choices for this first on-device build:
//!   1. **CPU DSP only.** Analysis runs through the pipeline runner
//!      (`pipeline::runner`, Plan v3): every kernel is a stage of
//!      `live_model.toml`, and the UI's profile, waterfall, scope, room
//!      calibration and capture export are tap consumers on the hop hook
//!      (see [`WireConsumers`]). It never constructs the wgpu `GpuYin`
//!      compute path, so a missing or limited Vulkan driver can't break
//!      launch.
//!   2. **glow (GLES/EGL) renderer**, not wgpu/Vulkan — forced via
//!      `NativeOptions.renderer`.
//!
//! Microphone capture needs the `RECORD_AUDIO` runtime permission. It is
//! declared in the manifest, but Android only grants it once the user
//! approves it. The app asks at launch (`permission`), and
//! [`start_audio_when_permitted`] opens the audio engine as soon as the
//! grant lands — on the spot when it is already held, otherwise from a
//! polling thread after the dialog. Until then the UI runs and says the
//! microphone is unavailable. See docs/android-build.md.
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
use crate::concurrency::{AnalysisState, ConcurrencyBridges, MicPermission, MicRequest, Telemetry};
use crate::pipeline::runner::{HopObserver, Runner, RunnerState};
use crate::pipeline::{AudioFrame, PipelineDefinition, Wire};
use crate::types::VocalProfile;
use crate::ui::DashboardApp;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use triple_buffer::Input;
use winit::platform::android::activity::AndroidApp;

use crate::frame::ANALYSIS_FRAME;

/// Fallback microphone rate when the input device can't be queried yet (e.g.
/// `RECORD_AUDIO` not granted at launch). 48 kHz is the near-universal Android
/// capture rate.
const FALLBACK_SAMPLE_RATE: f32 = 48_000.0;

/// How long a new generation waits for the previous runner to finish before
/// giving up and detaching it. The worker polls its shutdown flag every
/// `runner.poll_ms`, so this is ~100x the expected wait: long enough that it
/// never fires in practice, short enough that a wedged thread can't stop the
/// activity from relaunching.
const SHUTDOWN_GRACE: Duration = Duration::from_millis(500);

/// Poll interval while waiting out [`SHUTDOWN_GRACE`].
const SHUTDOWN_POLL: Duration = Duration::from_millis(5);

/// How often the grant state is re-read after the permission dialog was
/// shown. The poll runs until the grant lands or the generation is retired:
/// the user may grant it from Settings minutes later (Room → DIAGNOSTICS →
/// Open app settings) and expects audio to come up without a relaunch.
const PERMISSION_POLL: Duration = Duration::from_secs(1);

/// One `android_main` invocation's background resources, parked where the
/// *next* invocation can reach them.
struct Generation {
    /// Set to ask this generation's runner to return.
    shutdown: Arc<AtomicBool>,
    runner: Option<Runner>,
    /// Dropping this closes the AAudio input/output streams — the reason the
    /// engine lives here instead of on the parked `android_main` stack frame,
    /// which the new generation cannot reach. Shared with the thread that
    /// opens the engine once the microphone permission is granted.
    audio: Arc<Mutex<Option<AudioEngine>>>,
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
    let engine = match previous.audio.lock() {
        Ok(mut slot) => slot.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    };
    if engine.is_some() {
        crate::diagnostics::runtime::update_synthesizer_active(false);
    }
    drop(engine);

    // The spatial (TV-path) capture thread holds its own AudioRecord; it
    // checks this flag every 0.1 s chunk and releases the recorder itself.
    crate::spatial::request_shutdown();

    if let Some(mut runner) = previous.runner.take() {
        let stats = runner.stats();
        let deadline = Instant::now() + SHUTDOWN_GRACE;
        let done = |s: RunnerState| matches!(s, RunnerState::Stopped | RunnerState::Failed);
        while !done(stats.state()) && Instant::now() < deadline {
            thread::sleep(SHUTDOWN_POLL);
        }
        if done(stats.state()) {
            // Never join blind: a wedged worker would hang the relaunch.
            runner.stop();
        } else {
            log::warn!(
                "previous pipeline runner did not stop within {} ms; detaching it",
                SHUTDOWN_GRACE.as_millis()
            );
            runner.detach();
        }
    }
}

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    install_logger();
    log::info!("android_main: starting VoxLabs");

    // The Engineering Console's session log lives in the app's private
    // files dir; open it first so everything below can report into it.
    if let Some(files) = app.internal_data_path() {
        match crate::diagnostics::runtime::initialize(&files) {
            Ok(()) => {
                crate::diagnostics::install_panic_hook();
                // The bundle's `shared_model` block: the compiled-in atlas
                // files, their digests and their own release statements.
                crate::diagnostics::runtime::set_shared_model_snapshot(
                    serde_json::to_value(crate::atlas::data_provenance())
                        .unwrap_or(serde_json::Value::Null),
                );
            }
            Err(e) => log::error!("diagnostics store did not open: {e}"),
        }
    }
    crate::diagnostics::runtime::update_renderer_mode(
        crate::diagnostics::runtime::renderer_mode_name(),
    );

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

    // The analysis worker is the pipeline runner (Plan v3 Phase 1): the
    // Live Model stages per hop, plus the legacy per-frame tail. The runner
    // builds its pipeline on its own thread — the two inversion grids — so
    // audio is opened only once it is ready (bounded by the mode file's
    // init_timeout_ms), rather than letting the ring overflow meanwhile.
    let shutdown = Arc::new(AtomicBool::new(false));
    let telemetry_for_runner = bridges.telemetry.clone();
    let (runner, shell) = match PipelineDefinition::live_model() {
        Ok(def) => {
            let format = def.format(Some(input_sample_rate));
            if format.frame_samples != ANALYSIS_FRAME {
                // The capture export and the calibration pass count
                // ANALYSIS_FRAME-sample frames; a mode file that changes
                // the frame changes the study's file layout, so refuse.
                log::error!(
                    "live_model.toml frame_samples {} != ANALYSIS_FRAME {}; refusing to start",
                    format.frame_samples,
                    ANALYSIS_FRAME
                );
                telemetry_for_runner.set_analysis_state(AnalysisState::Unavailable);
                (None, None)
            } else {
                let tail = WireConsumers {
                    profile_tx,
                    ui_profile_tx,
                    spectrum_tx,
                    scope_tx,
                    telemetry: telemetry_for_runner.clone(),
                    calibrator: crate::math::RoomCalibrator::new(),
                    every: def.runner.legacy_frame_every_hops,
                };
                let init_timeout = Duration::from_millis(def.runner.init_timeout_ms);
                let poll = Duration::from_millis(def.runner.poll_ms);
                let (runner, shell) = Runner::spawn(
                    def,
                    format,
                    audio_rx,
                    Some(Box::new(tail)),
                    shutdown.clone(),
                );
                match runner.wait_ready(init_timeout, poll) {
                    RunnerState::Running => {
                        telemetry_for_runner.set_analysis_state(AnalysisState::Running)
                    }
                    RunnerState::Building => {
                        log::warn!(
                            "pipeline still building after {} ms; opening audio anyway",
                            init_timeout.as_millis()
                        );
                        telemetry_for_runner.set_analysis_state(AnalysisState::Running);
                    }
                    RunnerState::Failed | RunnerState::Stopped => {
                        log::error!("pipeline failed to start; live analysis unavailable");
                        telemetry_for_runner.set_analysis_state(AnalysisState::Unavailable);
                    }
                }
                (Some(runner), Some(shell))
            }
        }
        Err(e) => {
            log::error!("live_model.toml did not load: {e}");
            telemetry_for_runner.set_analysis_state(AnalysisState::Unavailable);
            (None, None)
        }
    };

    // cpal AAudio engine, opened as soon as RECORD_AUDIO is granted: now if
    // it already is, otherwise after the system dialog the app raises here.
    let audio: Arc<Mutex<Option<AudioEngine>>> = Arc::new(Mutex::new(None));
    start_audio_when_permitted(
        bridges.profile_rx,
        bridges.event_rx,
        bridges.audio_tx,
        bridges.telemetry.clone(),
        audio.clone(),
        shutdown.clone(),
        input_sample_rate,
    );

    // Park both handles where the *next* `android_main` can find them. This
    // invocation may never get the chance to clean up after itself.
    let generation = Generation {
        shutdown,
        runner,
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
            if let Some(shell) = shell {
                app.attach_pipeline(shell);
            }
            Ok(Box::new(app))
        }),
    ) {
        log::error!("eframe exited with error: {e:?}");
    }
}

/// Opens the audio engine once `RECORD_AUDIO` is granted. Held already:
/// opened on this thread before returning. Not held: the permission dialog
/// is raised, the UI shows the microphone-unavailable banner, and a thread
/// polls the grant state every [`PERMISSION_POLL`], opening the engine the
/// moment it lands, until the generation is retired. A failure to open with
/// the grant held stays fatal for audio, as before, and is logged.
fn start_audio_when_permitted(
    profile_rx: triple_buffer::Output<VocalProfile>,
    event_rx: rtrb::Consumer<crate::concurrency::EngineEvent>,
    audio_tx: rtrb::Producer<f32>,
    telemetry: Arc<Telemetry>,
    slot: Arc<Mutex<Option<AudioEngine>>>,
    shutdown: Arc<AtomicBool>,
    built_for_sample_rate: f32,
) {
    if crate::permission::has_record_audio() {
        telemetry.set_mic_permission(MicPermission::Granted);
        telemetry.set_mic_request(MicRequest::NotNeeded);
        crate::diagnostics::runtime::update_microphone_granted(true);
        log::info!("RECORD_AUDIO already granted; opening audio");
        open_audio(
            profile_rx,
            event_rx,
            audio_tx,
            telemetry,
            &slot,
            &shutdown,
            built_for_sample_rate,
        );
        return;
    }
    telemetry.set_mic_permission(MicPermission::NotGranted);
    telemetry.set_audio_unavailable(true);
    crate::diagnostics::runtime::update_microphone_granted(false);
    log::info!("RECORD_AUDIO not granted yet; asking the user");
    if crate::permission::request_record_audio() {
        telemetry.set_mic_request(MicRequest::Raised);
    } else {
        telemetry.set_mic_request(MicRequest::Failed);
        log::error!(
            "could not raise the microphone permission dialog; \
             grant it by hand: Room → DIAGNOSTICS → Open app settings → Permissions → Microphone"
        );
    }
    thread::spawn(move || {
        while !shutdown.load(Ordering::Relaxed) {
            thread::sleep(PERMISSION_POLL);
            if crate::permission::has_record_audio() {
                telemetry.set_mic_permission(MicPermission::Granted);
                crate::diagnostics::runtime::update_microphone_granted(true);
                log::info!("RECORD_AUDIO granted; opening audio");
                open_audio(
                    profile_rx,
                    event_rx,
                    audio_tx,
                    telemetry,
                    &slot,
                    &shutdown,
                    built_for_sample_rate,
                );
                return;
            }
        }
    });
}

/// Installs the Android logger wrapped in the in-app diagnostics tee, so the
/// Room screen can show the same lines logcat would. `set_boxed_logger`
/// fails on a relaunch in the same process (the first generation's logger is
/// still installed and already tees), which is fine to ignore.
fn install_logger() {
    let inner = android_logger::AndroidLogger::new(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    let _ = log::set_boxed_logger(Box::new(crate::diag::Tee { inner }));
    log::set_max_level(log::LevelFilter::Info);
}

/// Opens the cpal engine and parks it in the generation's slot. If the
/// generation was retired meanwhile the engine is dropped again at once, so
/// a stale launch can never hold the microphone.
fn open_audio(
    profile_rx: triple_buffer::Output<VocalProfile>,
    event_rx: rtrb::Consumer<crate::concurrency::EngineEvent>,
    audio_tx: rtrb::Producer<f32>,
    telemetry: Arc<Telemetry>,
    slot: &Mutex<Option<AudioEngine>>,
    shutdown: &AtomicBool,
    built_for_sample_rate: f32,
) {
    match AudioEngine::start(profile_rx, event_rx, audio_tx, telemetry.clone()) {
        Ok(engine) => {
            if let Some(rate) = query_input_sample_rate()
                && rate != built_for_sample_rate
            {
                // The runner and the analyzer were built for the rate read at
                // launch (the fallback when the grant was missing).
                log::warn!(
                    "input rate is {rate} Hz but analysis was built for {built_for_sample_rate} Hz; \
                     relaunch the app to rebuild at the right rate"
                );
            }
            match slot.lock() {
                Ok(mut s) => *s = Some(engine),
                Err(poisoned) => *poisoned.into_inner() = Some(engine),
            }
            if shutdown.load(Ordering::Relaxed) {
                // Retired while opening: release the microphone immediately.
                if let Ok(mut s) = slot.lock() {
                    s.take();
                }
                return;
            }
            telemetry.set_audio_unavailable(false);
            crate::diagnostics::runtime::update_synthesizer_active(true);
            log::info!("audio engine running");
        }
        Err(e) => {
            log::error!("audio engine failed to start: {e:?}; UI will run without audio");
            telemetry.set_audio_unavailable(true);
            crate::diagnostics::runtime::update_synthesizer_active(false);
        }
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

/// The tap consumers on the worker's hop hook: the profile the UI and the
/// synthesis reads, assembled from the wires (`pipeline::consumers`); the
/// waterfall from the `stft` tap; the oscilloscope from the frame; the
/// room-calibration pass (fed the pre-gate periodicity, since a
/// calibrating room with a voice in it must fail); the raw-capture
/// export. Every stage now runs once per hop; the calibration pass and
/// the capture mirror keep the frame cadence (`every` hops) so their
/// frame counts and file layout are unchanged.
struct WireConsumers {
    profile_tx: Input<VocalProfile>,
    ui_profile_tx: Input<VocalProfile>,
    spectrum_tx: Input<Vec<f32>>,
    scope_tx: Input<Vec<f32>>,
    telemetry: Arc<Telemetry>,
    calibrator: crate::math::RoomCalibrator,
    every: u64,
}

impl HopObserver for WireConsumers {
    fn on_hop(&mut self, frame: &AudioFrame, wires: &[Wire], hop: u64) {
        use crate::pipeline::consumers::{first, latest, profile_from_wires};
        use crate::pipeline::types::{F0Track, Spectrum};
        let samples = &frame.samples;
        let frame_cadence = hop.is_multiple_of(self.every);

        if frame_cadence {
            // Mirror the frame to the capture export (no-op unless a
            // capture is armed), one whole frame per `every` hops.
            crate::capture_log::push(samples);

            let (calibrating, calib_done) = self.telemetry.take_calibration_frame();
            if calibrating {
                let rms = crate::math::frame_rms(samples);
                // The estimator's own verdict, before the gates.
                let raw: Option<&F0Track> = first(wires);
                let yin_f0 = raw.filter(|t| t.voiced).map(|t| t.hz);
                self.calibrator.push(rms, yin_f0);
                if calib_done {
                    match self.calibrator.finish() {
                        Ok(cal) => {
                            crate::room::set_calibration(cal.ambient_rms, cal.interferer);
                            self.telemetry.set_calibration_result(Some((
                                cal.ambient_rms,
                                cal.interferer.map(|i| (i.f0_hz, i.rms)),
                            )));
                            crate::diagnostics::runtime::record_calibration(
                                crate::diagnostics::room_calibration_report(
                                    cal.ambient_rms,
                                    cal.interferer.map(|i| i.f0_hz),
                                ),
                            );
                            log::info!(
                                "room calibrated: ambient rms {:.5}, interferer {:?}",
                                cal.ambient_rms,
                                cal.interferer
                            );
                        }
                        Err(e) => {
                            log::warn!("room calibration failed: {e:?}");
                            self.telemetry.set_calibration_result(None);
                        }
                    }
                    self.calibrator = crate::math::RoomCalibrator::new();
                }
            }
        }

        if let Some(profile) = profile_from_wires(wires) {
            self.profile_tx.write(profile);
            self.ui_profile_tx.write(profile);
        }
        if let Some(spectrum) = latest::<Spectrum>(wires) {
            self.spectrum_tx.write(spectrum.magnitudes_db.clone());
        }
        self.scope_tx.write(samples.to_vec());

        // Heartbeat for the UI's staleness watch.
        self.telemetry.note_analysis_frame();
    }
}
