//! Coral's `note-detector.test.ts`, on the port.

use super::*;
use crate::config::ChoirDetectorConfig;

const SR: f32 = 44_100.0;
const NUM_BINS: usize = 1024;
const BIN_HZ: f32 = SR / (2.0 * NUM_BINS as f32);

fn opts(harmonic: bool) -> DetectorOptions {
    DetectorOptions {
        num_bins: NUM_BINS,
        sample_rate: SR,
        min_freq_hz: 50.0,
        max_freq_hz: 8000.0,
        harmonic,
        cfg: ChoirDetectorConfig::DEFAULT,
    }
}

fn spike(bin: usize, v: f32) -> Vec<f32> {
    let mut m = vec![0.0; NUM_BINS];
    m[bin] = v;
    m
}

fn harmonic_stack(f0: f32, count: usize) -> Vec<f32> {
    let mut m = vec![0.0; NUM_BINS];
    for k in 1..=count {
        let bin = ((k as f32 * f0) / BIN_HZ).round() as usize;
        if bin < NUM_BINS {
            m[bin] = 1.0 / k as f32;
        }
    }
    m
}

/// Coral's `note-detector.test.ts`, legacy path.
#[test]
fn legacy_bands_thresholds_polyphony_decay_and_reset() {
    let d = NoteDetector::new(opts(false)).unwrap();
    assert!(d.midi_min <= 69 && d.midi_max >= 69);
    let (lo, hi) = d.bin_range_for_midi(69).unwrap();
    let a4_bin = (440.0 / BIN_HZ).round() as i64;
    assert!(lo <= a4_bin && a4_bin <= hi);
    let half = 2f32.powf(1.0 / 24.0);
    for midi in [60, 69, 72, 96] {
        let f = 440.0 * 2f32.powf((midi - 69) as f32 / 12.0);
        let want_lo = ((f / half / BIN_HZ).floor() as i64).max(0);
        let want_hi = ((f * half / BIN_HZ).ceil() as i64).min(NUM_BINS as i64 - 1);
        assert_eq!(d.bin_range_for_midi(midi), Some((want_lo, want_hi)));
    }
    let narrow = NoteDetector::new(DetectorOptions {
        min_freq_hz: 200.0,
        max_freq_hz: 2000.0,
        ..opts(false)
    })
    .unwrap();
    assert!(narrow.bin_range_for_midi(45).is_none());
    assert!(narrow.bin_range_for_midi(93).is_some());
    let mut d = NoteDetector::new(opts(false)).unwrap();
    assert!(d.analyze(&vec![0.0; NUM_BINS]).unwrap().is_empty());
    let mut above = NoteDetector::new(opts(false)).unwrap();
    assert!(
        above
            .analyze(&spike(97, 10f32.powf(-49.5 / 20.0)))
            .unwrap()
            .contains(&96)
    );
    let mut below = NoteDetector::new(opts(false)).unwrap();
    assert!(
        !below
            .analyze(&spike(97, 10f32.powf(-50.5 / 20.0)))
            .unwrap()
            .contains(&96)
    );
    let mut d = NoteDetector::new(opts(false)).unwrap();
    let a = d.analyze(&spike(97, 1.0)).unwrap().to_vec();
    assert!(a.contains(&96) && !a.contains(&95) && !a.contains(&97));
    let mut m = vec![0.0; NUM_BINS];
    m[97] = 1.0;
    m[122] = 1.0;
    m[145] = 1.0;
    let mut d = NoteDetector::new(opts(false)).unwrap();
    let a = d.analyze(&m).unwrap().to_vec();
    for n in [96, 100, 103] {
        assert!(a.contains(&n), "{a:?}");
    }
    let mut d = NoteDetector::new(opts(false)).unwrap();
    d.analyze(&spike(97, 0.5)).unwrap();
    let silent = vec![0.0; NUM_BINS];
    let mut frames = 0;
    while frames < 200 {
        if !d.analyze(&silent).unwrap().contains(&96) {
            break;
        }
        frames += 1;
    }
    assert!(frames > 55 && frames < 70, "{frames}");
    let mut d = NoteDetector::new(opts(false)).unwrap();
    assert!(d.analyze(&spike(97, 1.0)).unwrap().contains(&96));
    d.reset();
    assert!(d.analyze(&silent).unwrap().is_empty());
    assert!(
        NoteDetector::new(DetectorOptions {
            num_bins: 0,
            ..opts(false)
        })
        .is_err()
    );
    assert!(
        NoteDetector::new(DetectorOptions {
            sample_rate: -1.0,
            ..opts(false)
        })
        .is_err()
    );
    assert!(
        NoteDetector::new(DetectorOptions {
            min_freq_hz: 0.0,
            ..opts(false)
        })
        .is_err()
    );
    assert!(
        NoteDetector::new(DetectorOptions {
            max_freq_hz: 50.0,
            ..opts(false)
        })
        .is_err()
    );
    assert!(d.analyze(&[0.0; 5]).is_err());
}

/// Coral's `note-detector.test.ts`, harmonic path.
#[test]
fn harmonic_stack_reads_as_one_fundamental_and_flat_spectra_are_rejected() {
    let mut d = NoteDetector::new(opts(true)).unwrap();
    let a = d.analyze(&harmonic_stack(220.0, 7)).unwrap().to_vec();
    assert!(a.contains(&57), "{a:?}");
    assert!(!a.contains(&69), "{a:?}");
    let conf = d.last_confidences();
    assert_eq!(conf.iter().map(|c| c.0).collect::<Vec<_>>(), a);
    assert!(conf.iter().all(|c| c.1 >= 1.0));
    let flat = vec![0.5f32; NUM_BINS];
    let mut legacy = NoteDetector::new(opts(false)).unwrap();
    assert!(!legacy.analyze(&flat).unwrap().is_empty());
    let mut harmonic = NoteDetector::new(opts(true)).unwrap();
    assert!(harmonic.analyze(&flat).unwrap().is_empty());
    assert!(legacy.harmonic_bins_for_midi(57).is_none());
    let ranges = d.harmonic_bins_for_midi(57).unwrap();
    assert_eq!(ranges.len(), 9);
    assert_eq!(ranges[0], d.bin_range_for_midi(57).unwrap());
    d.reset();
    assert!(d.analyze(&vec![0.0; NUM_BINS]).unwrap().is_empty());
    let bad = ChoirDetectorConfig {
        harmonic_count: 0,
        ..ChoirDetectorConfig::DEFAULT
    };
    assert!(
        NoteDetector::new(DetectorOptions {
            cfg: bad,
            ..opts(true)
        })
        .is_err()
    );
    assert!(
        NoteDetector::new(DetectorOptions {
            cfg: bad,
            ..opts(false)
        })
        .is_ok()
    );
}
