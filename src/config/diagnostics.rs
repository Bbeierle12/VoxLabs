//! Engineering-console (diagnostics) configuration: the offline assistant's
//! rule thresholds and health penalties, the local store's caps, the
//! refinement-form validation ranges, and the runtime's sampling cadence.
//!
//! Ported as-is from Vocal Tract Lab 0.10.0's `DiagnosticsCore.kt`,
//! `DiagnosticStore.kt` and `DiagnosticsRuntime.kt`; every number those
//! files carry inline is a field here so the console's behaviour is in
//! `pipeline.toml` like every other stage's.

use super::stage_config;

stage_config! {
    /// Diagnostics: offline assistant rules, store limits, form validation.
    pub struct DiagnosticsConfig, section = "diagnostics" {
        /// Minimum spacing between `derived_state_sample` events, ms; a
        /// change in the dropped-frame count bypasses it. Range: 100..=5000.
        sample_interval_ms: u32 = 500,
        /// Events the console lists under "Recent session events".
        /// Range: 10..=200.
        recent_events_shown: usize = 30,
        /// DSP time above this fraction of the frame budget is "budget
        /// pressure" (a warning); above the whole budget is a missed
        /// deadline (an error). Range: 0.25..=0.9.
        budget_pressure_fraction: f32 = 0.5,
        /// Dropped frames at or above this make `queue_drops` an error
        /// instead of a warning. Range: 1..=100.
        drops_error_threshold: u32 = 10,
        /// Voiced frames with f0 confidence below this raise
        /// `low_f0_confidence`. Range: 0.3..=0.9.
        f0_confidence_warn: f32 = 0.55,
        /// Voiced frames with SNR below this, dB, raise
        /// `low_signal_to_noise`. Range: 0.0..=20.0.
        snr_low_db: f32 = 6.0,
        /// f0 below this, Hz, is outside the normal analysis range.
        /// Range: 20.0..=100.0.
        f0_unusual_min_hz: f32 = 50.0,
        /// f0 above this, Hz, is outside the normal analysis range.
        /// Range: 800.0..=3000.0.
        f0_unusual_max_hz: f32 = 1500.0,
        /// Mean formant standard deviation above this, Hz, raises
        /// `uncertain_resonances`. Range: 50.0..=1000.0.
        formant_std_warn_hz: f32 = 250.0,
        /// Tract confidence below this is an error (`very_low_tract_confidence`).
        /// Range: 0.0..=tract_confidence_warn.
        tract_confidence_error: f32 = 0.15,
        /// Tract confidence below this is a warning (`low_tract_confidence`).
        /// Range: tract_confidence_error..=0.9.
        tract_confidence_warn: f32 = 0.35,
        /// Relative airway-area standard deviation above this raises
        /// `large_area_uncertainty`. Range: 0.1..=1.0.
        area_std_warn: f32 = 0.65,
        /// Health-score points removed per ERROR finding. Range: 1..=100.
        penalty_error: u32 = 25,
        /// Health-score points removed per WARNING finding. Range: 1..=100.
        penalty_warning: u32 = 10,
        /// Health-score points removed per INFO finding. Range: 0..=100.
        penalty_info: u32 = 2,
        /// Health score is clamped to 0..=this. Range: exactly 100.
        health_max: u32 = 100,
        /// Lowest expected F0 a correction may propose, Hz. Range: 20.0..=100.0.
        expected_f0_min_hz: f32 = 40.0,
        /// Highest expected F0 a correction may propose, Hz. Range: 1000.0..=4000.0.
        expected_f0_max_hz: f32 = 2000.0,
        /// Most expected resonances a correction may list (R1–R6). Range: 1..=8.
        expected_resonances_max: usize = 6,
        /// Lowest expected resonance, Hz. Range: 50.0..=200.0.
        resonance_min_hz: f32 = 80.0,
        /// Highest expected resonance, Hz. Range: 5000.0..=20000.0.
        resonance_max_hz: f32 = 10000.0,
        /// Session event file cap, bytes; appends stop past it. Range: 1 MiB..=64 MiB.
        max_session_bytes: usize = 4194304,
        /// Refinement file cap, bytes; appends fail past it. Range: 1 MiB..=64 MiB.
        max_refinement_bytes: usize = 4194304,
        /// Newest session files kept; older ones are pruned at startup.
        /// Range: 1..=100.
        max_session_files: usize = 20,
        /// Events carried in an export bundle (and the most `recent_events`
        /// may return). Range: 100..=10000.
        max_exported_events: usize = 1500,
        /// Refinement records carried in an export bundle. Range: 10..=5000.
        max_exported_refinements: usize = 500,
        /// Event category is truncated to this many characters. Range: 8..=100.
        category_max_chars: usize = 40,
        /// Event code is truncated to this many characters. Range: 8..=200.
        code_max_chars: usize = 80,
        /// Event message and each evidence value are truncated to this many
        /// characters. Range: 50..=2000.
        message_max_chars: usize = 500,
        /// Characters of the record/session UUID shown to the user.
        /// Range: 4..=32.
        short_id_chars: usize = 8,
        /// Stack frames kept in a crash event's evidence. Range: 1..=32.
        crash_top_frames: usize = 8,
        /// Pitch self-test: fixture tone, Hz. Range: 60.0..=400.0.
        self_test_tone_hz: f32 = 110.0,
        /// Pitch self-test: accepted low bound, Hz. Range: below tone.
        self_test_tone_min_hz: f32 = 105.0,
        /// Pitch self-test: accepted high bound, Hz. Range: above tone.
        self_test_tone_max_hz: f32 = 116.0,
        /// Pitch self-test: fixture amplitude, linear. Range: 0.05..=0.9.
        self_test_tone_amplitude: f32 = 0.3,
        /// Noise-floor self-test: quiet frame RMS. Range: 1e-5..=1e-2.
        self_test_quiet_rms: f32 = 0.001,
        /// Noise-floor self-test: quiet frames pushed before measuring.
        /// Range: noise_floor.ring_min..=noise_floor.ring_len.
        self_test_quiet_frames: usize = 40,
        /// Noise-floor self-test: loud frame RMS. Range: 0.01..=1.0.
        self_test_loud_rms: f32 = 0.1,
        /// Noise-floor self-test: SNR the loud frame must reach, dB.
        /// Range: 6.0..=40.0.
        self_test_min_snr_db: f32 = 30.0,
        /// Synthesis self-test: samples rendered. Range: 64..=8192.
        self_test_synth_samples: usize = 512,
        /// Synthesis self-test: a sample above this counts as sound.
        /// Range: 1e-9..=1e-2.
        self_test_synth_silence: f32 = 1e-5,
        /// Synthesis self-test: sample rate the bank renders at, Hz.
        /// Range: 8000.0..=96000.0.
        self_test_synth_rate_hz: f32 = 48000.0,
        /// Synthesis self-test: glide time handed to the bank, ms. Range: 1.0..=500.0.
        self_test_synth_glide_ms: f32 = 20.0,
        /// Console toast lifetime, seconds. Range: 1.0..=10.0.
        toast_secs: f64 = 3.5,
    }
}
