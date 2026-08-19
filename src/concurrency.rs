use crate::types::VocalProfile;
use crossbeam_utils::CachePadded;
use rtrb::{Consumer, Producer, RingBuffer};
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU32, Ordering};
use triple_buffer::{Input, Output, TripleBuffer};

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
        }
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
