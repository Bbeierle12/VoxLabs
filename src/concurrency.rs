use crate::types::VocalProfile;
use crossbeam_utils::CachePadded;
use rtrb::{Consumer, Producer, RingBuffer};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};
use triple_buffer::{Input, Output, TripleBuffer};

/// Room-calibration lifecycle, one atomic byte in [`Telemetry`]. The UI
/// requests a pass; the analysis thread runs it and reports back.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum CalibState {
    Idle = 0,
    Running = 1,
    Done = 2,
    /// A voice (or TV) talked during the pass — discarded, retry in silence.
    FailedVoice = 3,
}

impl CalibState {
    fn from_u8(raw: u8) -> Self {
        match raw {
            1 => Self::Running,
            2 => Self::Done,
            3 => Self::FailedVoice,
            _ => Self::Idle,
        }
    }
}

/// Liveness of the analysis thread, as seen by the UI.
///
/// The analysis path runs on its own thread and publishes through lock-free
/// buffers, so its death is otherwise indistinguishable from a silent
/// microphone: the last profile just stops changing. This enum is stored as a
/// single atomic byte in [`Telemetry`] so every entry point can report which
/// of the three "no data" cases it is in, and the UI can say so instead of
/// freezing on a stale readout.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum AnalysisState {
    /// Thread spawned; the engine has not finished initializing yet.
    Starting = 0,
    /// Initialized and producing frames.
    Running = 1,
    /// Never started — initialization failed (on desktop: no usable GPU
    /// adapter). No frames will ever arrive.
    Unavailable = 2,
    /// Ran, then stopped: the loop panicked or hit unrecoverable frame errors.
    /// No further frames will arrive until the app is restarted.
    Stopped = 3,
}

impl AnalysisState {
    /// Decodes the atomic representation. Unknown bytes read as `Starting`
    /// rather than panicking — this is a status flag, not a data channel.
    fn from_u8(raw: u8) -> Self {
        match raw {
            1 => Self::Running,
            2 => Self::Unavailable,
            3 => Self::Stopped,
            _ => Self::Starting,
        }
    }
}

pub struct Telemetry {
    pub xruns: CachePadded<AtomicU32>,
    pub consumed_frames: CachePadded<AtomicU32>,
    /// Latest microphone input RMS as `f32::to_bits`, written once per input
    /// callback. Drives the Capture screen's LEVEL readout and waveform.
    pub input_rms: CachePadded<AtomicU32>,
    /// [`AnalysisState`] discriminant. Written by the analysis thread, read by
    /// the UI; use [`Telemetry::analysis_state`] / [`Telemetry::set_analysis_state`].
    analysis_state: CachePadded<AtomicU8>,
    /// Monotonic count of analysis frames published. Wrapping is harmless: the
    /// UI only compares it with the previous value it saw, to detect a loop
    /// that is alive but no longer producing (see `DashboardApp::watch_engine`).
    pub analysis_frames: CachePadded<AtomicU32>,
    /// cpal stream errors reported to the input/output error callbacks. The
    /// callbacks cannot render anything themselves, so they count here and the
    /// UI turns a change into a banner.
    pub input_stream_errors: CachePadded<AtomicU32>,
    pub output_stream_errors: CachePadded<AtomicU32>,
    /// Set when the audio engine could not be started at all — on Android,
    /// almost always a missing `RECORD_AUDIO` grant. Distinct from a stream
    /// *error*: there is no stream. Without this the UI sits in SEARCHING
    /// forever and the only explanation is a logcat line the user cannot see.
    audio_unavailable: CachePadded<AtomicBool>,
    /// Room-calibration control plane: frames still to collect (UI arms it,
    /// the analysis loop drains it), lifecycle state, and the result summary
    /// (RMS values as f32 bits; interferer f0 in Hz, 0 = none). Relaxed
    /// throughout — control flags, nothing ordered behind them.
    calib_frames_left: CachePadded<AtomicU32>,
    calib_state: CachePadded<AtomicU8>,
    calib_ambient_rms: CachePadded<AtomicU32>,
    calib_interferer_f0: CachePadded<AtomicU32>,
    calib_interferer_rms: CachePadded<AtomicU32>,
    /// Microphone permission as the shell last saw it ([`MicPermission`]),
    /// and whether the system dialog could be raised ([`MicRequest`]).
    /// Written by the Android entry point, read by the DIAGNOSTICS card.
    mic_permission: CachePadded<AtomicU8>,
    mic_request: CachePadded<AtomicU8>,
}

/// RECORD_AUDIO grant state as the shell observes it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum MicPermission {
    /// Not checked on this target (desktop, web).
    Unknown,
    NotGranted,
    Granted,
}

/// Whether the shell managed to raise the system permission dialog.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum MicRequest {
    NotNeeded,
    Raised,
    Failed,
}

impl MicPermission {
    const ALL: &'static [MicPermission] = &[
        MicPermission::Unknown,
        MicPermission::NotGranted,
        MicPermission::Granted,
    ];
}

impl MicRequest {
    const ALL: &'static [MicRequest] = &[
        MicRequest::NotNeeded,
        MicRequest::Raised,
        MicRequest::Failed,
    ];
}

impl Telemetry {
    pub fn new() -> Self {
        Self {
            xruns: CachePadded::new(AtomicU32::new(0)),
            consumed_frames: CachePadded::new(AtomicU32::new(0)),
            input_rms: CachePadded::new(AtomicU32::new(0)),
            analysis_state: CachePadded::new(AtomicU8::new(AnalysisState::Starting as u8)),
            analysis_frames: CachePadded::new(AtomicU32::new(0)),
            input_stream_errors: CachePadded::new(AtomicU32::new(0)),
            output_stream_errors: CachePadded::new(AtomicU32::new(0)),
            audio_unavailable: CachePadded::new(AtomicBool::new(false)),
            calib_frames_left: CachePadded::new(AtomicU32::new(0)),
            calib_state: CachePadded::new(AtomicU8::new(CalibState::Idle as u8)),
            calib_ambient_rms: CachePadded::new(AtomicU32::new(0)),
            calib_interferer_f0: CachePadded::new(AtomicU32::new(0)),
            calib_interferer_rms: CachePadded::new(AtomicU32::new(0)),
            mic_permission: CachePadded::new(AtomicU8::new(MicPermission::Unknown as u8)),
            mic_request: CachePadded::new(AtomicU8::new(MicRequest::NotNeeded as u8)),
        }
    }

    pub fn mic_permission(&self) -> MicPermission {
        MicPermission::ALL
            .get(self.mic_permission.load(Ordering::Relaxed) as usize)
            .copied()
            .unwrap_or(MicPermission::Unknown)
    }

    pub fn set_mic_permission(&self, p: MicPermission) {
        self.mic_permission.store(p as u8, Ordering::Relaxed);
    }

    pub fn mic_request(&self) -> MicRequest {
        MicRequest::ALL
            .get(self.mic_request.load(Ordering::Relaxed) as usize)
            .copied()
            .unwrap_or(MicRequest::NotNeeded)
    }

    pub fn set_mic_request(&self, r: MicRequest) {
        self.mic_request.store(r as u8, Ordering::Relaxed);
    }

    /// UI: arm a calibration pass of `frames` analysis frames.
    pub fn start_calibration(&self, frames: u32) {
        self.calib_frames_left.store(frames, Ordering::Relaxed);
        self.calib_state
            .store(CalibState::Running as u8, Ordering::Relaxed);
    }

    /// Analysis loop: claim the next frame of an armed pass. Returns true
    /// while calibration frames remain (this frame is part of the pass) and
    /// whether it was the final one.
    pub fn take_calibration_frame(&self) -> (bool, bool) {
        let prev = self
            .calib_frames_left
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_sub(1))
            .unwrap_or(0);
        (prev > 0, prev == 1)
    }

    pub fn calib_state(&self) -> CalibState {
        CalibState::from_u8(self.calib_state.load(Ordering::Relaxed))
    }

    pub fn calib_frames_left(&self) -> u32 {
        self.calib_frames_left.load(Ordering::Relaxed)
    }

    /// Analysis loop: publish a finished pass (or its failure).
    pub fn set_calibration_result(
        &self,
        result: Option<(f32, Option<(f32, f32)>)>, // (ambient_rms, (f0, rms))
    ) {
        match result {
            Some((ambient, interferer)) => {
                self.calib_ambient_rms
                    .store(ambient.to_bits(), Ordering::Relaxed);
                let (f0, irms) = interferer.unwrap_or((0.0, 0.0));
                self.calib_interferer_f0
                    .store(f0.to_bits(), Ordering::Relaxed);
                self.calib_interferer_rms
                    .store(irms.to_bits(), Ordering::Relaxed);
                self.calib_state
                    .store(CalibState::Done as u8, Ordering::Relaxed);
            }
            None => {
                self.calib_state
                    .store(CalibState::FailedVoice as u8, Ordering::Relaxed);
            }
        }
    }

    /// UI: the published summary — (ambient RMS, interferer (f0 Hz, RMS)).
    pub fn calibration_summary(&self) -> (f32, Option<(f32, f32)>) {
        let ambient = f32::from_bits(self.calib_ambient_rms.load(Ordering::Relaxed));
        let f0 = f32::from_bits(self.calib_interferer_f0.load(Ordering::Relaxed));
        let irms = f32::from_bits(self.calib_interferer_rms.load(Ordering::Relaxed));
        (ambient, (f0 > 0.0).then_some((f0, irms)))
    }

    /// Whether the audio engine failed to start. Set once at startup by
    /// whichever entry point owns the engine.
    pub fn audio_unavailable(&self) -> bool {
        self.audio_unavailable.load(Ordering::Relaxed)
    }

    pub fn set_audio_unavailable(&self, unavailable: bool) {
        self.audio_unavailable.store(unavailable, Ordering::Relaxed);
    }

    /// Current analysis-thread liveness. `Relaxed` throughout: these flags
    /// order nothing else — the profile itself travels through the triple
    /// buffer, which does its own synchronization.
    pub fn analysis_state(&self) -> AnalysisState {
        AnalysisState::from_u8(self.analysis_state.load(Ordering::Relaxed))
    }

    pub fn set_analysis_state(&self, state: AnalysisState) {
        self.analysis_state.store(state as u8, Ordering::Relaxed);
    }

    /// Called by an analysis loop once per published profile — the heartbeat
    /// the UI's staleness watch counts.
    pub fn note_analysis_frame(&self) {
        self.analysis_frames.fetch_add(1, Ordering::Relaxed);
    }
}

pub enum EngineEvent {
    /// Reserved: emergency mute (UI not yet wired to send it).
    #[allow(dead_code)]
    PanicFlag,
    /// Reserved: reset oscillator phases (UI not yet wired to send it).
    #[allow(dead_code)]
    Reset,
    SetHarmonicCount(usize),
    SetDeltaF(f32),
}

pub struct ConcurrencyBridges {
    // Profiler updates (Analysis -> Synthesis)
    pub profile_tx: Input<VocalProfile>,
    pub profile_rx: Output<VocalProfile>,

    // Profiler updates (Analysis -> UI)
    pub ui_profile_tx: Input<VocalProfile>,
    pub ui_profile_rx: Output<VocalProfile>,

    // Audio input (Audio -> Analysis)
    pub audio_tx: Producer<f32>,
    pub audio_rx: Consumer<f32>,

    // Spectrogram magnitudes, one FFT frame in dB (Analysis -> UI waterfall)
    pub spectrum_tx: Input<Vec<f32>>,
    pub spectrum_rx: Output<Vec<f32>>,

    // Raw waveform, most recent analysis frame (Analysis -> UI oscilloscope)
    pub scope_tx: Input<Vec<f32>>,
    pub scope_rx: Output<Vec<f32>>,

    // Event updates (UI -> Synthesis)
    pub event_tx: Producer<EngineEvent>,
    pub event_rx: Consumer<EngineEvent>,

    // Telemetry (Synthesis -> UI/Analysis)
    pub telemetry: Arc<Telemetry>,
}

impl Default for ConcurrencyBridges {
    fn default() -> Self {
        Self::new()
    }
}

impl ConcurrencyBridges {
    pub fn new() -> Self {
        let (profile_tx, profile_rx) = TripleBuffer::new(&VocalProfile::default()).split();
        let (ui_profile_tx, ui_profile_rx) = TripleBuffer::new(&VocalProfile::default()).split();
        let (audio_tx, audio_rx) = RingBuffer::new(8192);
        let (event_tx, event_rx) = RingBuffer::new(256);
        let (spectrum_tx, spectrum_rx) = TripleBuffer::new(&vec![
            crate::spectrogram::DB_FLOOR;
            crate::spectrogram::N_BINS
        ])
        .split();
        // Raw scope buffer sized to one analysis frame; the UI reads its length
        // generically, so the exact size here is only the pre-signal default.
        let (scope_tx, scope_rx) = TripleBuffer::new(&vec![0.0f32; 2048]).split();
        let telemetry = Arc::new(Telemetry::new());

        Self {
            profile_tx,
            profile_rx,
            ui_profile_tx,
            ui_profile_rx,
            audio_tx,
            audio_rx,
            spectrum_tx,
            spectrum_rx,
            scope_tx,
            scope_rx,
            event_tx,
            event_rx,
            telemetry,
        }
    }
}
