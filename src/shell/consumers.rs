//! The tap consumers on the worker's hop hook: the profile the UI and the
//! synthesis read, assembled from the wires (`pipeline::consumers`); the
//! waterfall from the newest `Spectrum` (max-pooled to the waterfall's
//! bin count when a mode's FFT differs); the oscilloscope from the
//! frame; the room-calibration pass (fed the pre-gate periodicity, since
//! a calibrating room with a voice in it must fail); the raw-capture
//! export. Every stage runs once per hop; the calibration pass and the
//! capture mirror keep the frame cadence (`every` hops) and the capture
//! mirror writes only the `hop × every` new samples, so the capture file
//! stays contiguous audio whatever the mode's frame size.

use std::sync::Arc;

use triple_buffer::Input;

use crate::concurrency::Telemetry;
use crate::pipeline::runner::HopObserver;
use crate::pipeline::{AudioFrame, Wire};
use crate::types::VocalProfile;

pub struct WireConsumers {
    profile_tx: Input<VocalProfile>,
    ui_profile_tx: Input<VocalProfile>,
    spectrum_tx: Input<Vec<f32>>,
    scope_tx: Input<Vec<f32>>,
    telemetry: Arc<Telemetry>,
    calibrator: crate::math::RoomCalibrator,
    every: u64,
    hop: usize,
    /// Reused when the mode's spectrum has more bins than the waterfall.
    pooled: Vec<f32>,
}

impl WireConsumers {
    pub fn new(
        profile_tx: Input<VocalProfile>,
        ui_profile_tx: Input<VocalProfile>,
        spectrum_tx: Input<Vec<f32>>,
        scope_tx: Input<Vec<f32>>,
        telemetry: Arc<Telemetry>,
        every: u64,
        hop: usize,
    ) -> Self {
        Self {
            profile_tx,
            ui_profile_tx,
            spectrum_tx,
            scope_tx,
            telemetry,
            calibrator: crate::math::RoomCalibrator::new(),
            every: every.max(1),
            hop,
            pooled: vec![crate::spectrogram::DB_FLOOR; crate::spectrogram::N_BINS],
        }
    }

    /// A mode switch keeps the observer but changes the cadence.
    pub fn set_cadence(&mut self, every: u64, hop: usize) {
        self.every = every.max(1);
        self.hop = hop;
    }
}

/// Max-pools `src` (dB per bin) onto `dst.len()` bins over the same
/// Nyquist span. Equal lengths copy.
pub fn pool_spectrum_db(src: &[f32], dst: &mut [f32]) {
    if src.len() == dst.len() {
        dst.copy_from_slice(src);
        return;
    }
    let n = dst.len();
    for (i, d) in dst.iter_mut().enumerate() {
        let lo = (i * src.len()) / n;
        let hi = (((i + 1) * src.len()) / n).max(lo + 1).min(src.len());
        *d = src[lo..hi]
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
    }
}

impl HopObserver for WireConsumers {
    fn on_hop(&mut self, frame: &AudioFrame, wires: &[Wire], hop: u64) {
        use crate::pipeline::consumers::{first, latest, profile_from_wires};
        use crate::pipeline::types::{F0Track, Spectrum};
        let samples = &frame.samples;
        let frame_cadence = hop.is_multiple_of(self.every);

        if frame_cadence {
            // The capture export gets the new audio since the last mirror
            // (no-op unless a capture is armed).
            let new = (self.hop * self.every as usize).min(samples.len());
            crate::capture_log::push(&samples[samples.len() - new..]);

            let (calibrating, calib_done) = self.telemetry.take_calibration_frame();
            if calibrating {
                let rms = crate::math::frame_rms(samples);
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
            pool_spectrum_db(&spectrum.magnitudes_db, &mut self.pooled);
            self.spectrum_tx.write(self.pooled.clone());
        }
        self.scope_tx.write(samples.to_vec());
        self.telemetry.note_analysis_frame();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pooling_takes_the_max_per_destination_bin_and_copies_equal_lengths() {
        let src: Vec<f32> = (0..8).map(|i| i as f32).collect();
        let mut dst = [0.0f32; 4];
        pool_spectrum_db(&src, &mut dst);
        assert_eq!(dst, [1.0, 3.0, 5.0, 7.0]);
        let mut same = [0.0f32; 8];
        pool_spectrum_db(&src, &mut same);
        assert_eq!(same.to_vec(), src);
    }
}
