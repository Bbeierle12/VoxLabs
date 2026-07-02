use crate::types::VocalProfile;
use crossbeam_utils::CachePadded;
use rtrb::{Consumer, Producer, RingBuffer};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32};
use triple_buffer::{Input, Output, TripleBuffer};

pub struct Telemetry {
    pub xruns: CachePadded<AtomicU32>,
    pub consumed_frames: CachePadded<AtomicU32>,
    /// Latest microphone input RMS as `f32::to_bits`, written once per input
    /// callback. Drives the Capture screen's LEVEL readout and waveform.
    pub input_rms: CachePadded<AtomicU32>,
    /// Reserved: engine liveness flag for a future UI status indicator.
    #[allow(dead_code)]
    pub alive: CachePadded<AtomicBool>,
}

impl Telemetry {
    pub fn new() -> Self {
        Self {
            xruns: CachePadded::new(AtomicU32::new(0)),
            consumed_frames: CachePadded::new(AtomicU32::new(0)),
            input_rms: CachePadded::new(AtomicU32::new(0)),
            alive: CachePadded::new(AtomicBool::new(true)),
        }
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
