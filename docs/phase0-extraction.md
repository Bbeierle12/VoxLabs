# Phase 0 (d) — numeric literals extracted from the DSP modules

Plan v3 §5 Phase 0: every numeric literal in `tract.rs` and the DSP modules
moved into a `StageConfig` struct (`src/config/`) mirrored by `pipeline.toml`,
or into a named definitional constant (`src/config/consts.rs`). Exceptions:
`0`, `1`, `-1`, π, e. No behavior change: each `DEFAULT` is the literal that was
in the code.

Modules covered: `math.rs`, `metrics.rs`, `fach.rs`, `spectrogram.rs`,
`synthesis.rs`, `frame.rs`, `spatial.rs` (the cross-target processor),
`analysis.rs`, `tract.rs`, the scalar constants of `tract_data.rs`, the
resampler in `audio_file.rs`, and the synthesis glide constant in `audio.rs`.

Not extracted, by design:

- the published 44-element Story 2018 basis tables and the Table II vowel
  anchors in `tract_data.rs` — data, kept in the data module;
- the Android JNI capture thread in `spatial.rs` (`#[cfg(target_os =
  "android")] mod capture`) — platform capture I/O, not a pipeline stage
  (Plan v3 D6), in the same category as `device_probe.rs`;
- test modules — expected values are the tests.

Gate: a literal scanner over the nine DSP modules (comments, strings,
`#[cfg(test)]` and `#[cfg(target_os = "android")]` items excluded) reports zero
literals other than the exceptions. `cargo test config::` proves
`pipeline.toml` and the code defaults agree, key for key.

Old locations are line numbers at commit `470e4c8` (the tree before this
step). "Replaced by" names the config field(s) or constant(s) that now carry
the value; `TAU` is `std::f32::consts::TAU`.

| File | Old line: literal | Replaced by |
|---|---|---|
| `src/math.rs` | 8: 1024 | `YIN.window` |
| `src/math.rs` | 11: 50.0, 12: 1000.0, 13: 0.12 | `YIN.f0_min_hz`, `YIN.f0_max_hz`, `YIN.threshold` |
| `src/math.rs` | 46: 1e-9 | `LPC.levinson_error_floor` |
| `src/math.rs` | 83: 3 | `N_FORMANTS` |
| `src/math.rs` | 87: 3 | `N_FORMANTS` |
| `src/math.rs` | 89: 3 | `FORMANT.min_coefficients` |
| `src/math.rs` | 97: 50, 98: 1e-9 | `FORMANT.aberth_max_iterations`, `FORMANT.aberth_epsilon` |
| `src/math.rs` | 102: 2.0 | `std::f64::consts::TAU` |
| `src/math.rs` | 119: 90.0, 119: 5000.0, 119: 500.0 | `FORMANT.band_lo_hz`, `FORMANT.band_hi_hz`, `FORMANT.max_bandwidth_hz` |
| `src/math.rs` | 151: 2.0, 152: 1e-6 | `TWO`, `YIN.parabolic_flat_eps` |
| `src/math.rs` | 155: 2.0 | `TWO` |
| `src/math.rs` | 196: 3 | `YIN.min_max_lag` |
| `src/math.rs` | 199: 2 | `YIN.min_tau` |
| `src/math.rs` | 303: 2, 304: 2 | `YIN_WINDOW.min`, `TWO_USIZE`, `YIN.min_window` |
| `src/math.rs` | 310: 3 | `YIN.min_max_lag` |
| `src/math.rs` | 338: 2.0, 339: 1e-6, 340: 2.0 | `TWO`, `LPC.sinc_center_eps` |
| `src/math.rs` | 342: 2.0 | `TAU` |
| `src/math.rs` | 344: 0.54, 344: 0.46, 344: 2.0 | `HAMMING_A0`, `HAMMING_A1`, `TAU` |
| `src/math.rs` | 348: 1e-9 | `LPC.fir_gain_eps` |
| `src/math.rs` | 363: 0.45, 364: 31, 365: 2 | `LPC.decimation_cutoff_of_nyquist`, `LPC.decimation_fir_taps`, `TWO_USIZE` |
| `src/math.rs` | 406: 0.54, 406: 0.46, 406: 2.0 | `HAMMING_A0`, `HAMMING_A1`, `TAU` |
| `src/math.rs` | 411: 1e-9 | `LPC.silence_energy` |
| `src/math.rs` | 436: 32 | `HARMONICS.min_frame_samples` |
| `src/math.rs` | 446: 0.5, 446: 0.5, 446: 2.0 | `HANN_A0`, `TAU` |
| `src/math.rs` | 450: 2.0 | `TWO` |
| `src/math.rs` | 452: 2.0 | `TWO` |
| `src/math.rs` | 459: 2.0, 460: 2.0 | `TAU`, `TWO` |
| `src/math.rs` | 483: 64 | `HNR.min_frame_samples` |
| `src/math.rs` | 488: 2, 488: 2 | `HNR.min_period_samples`, `HNR.period_margin_samples` |
| `src/math.rs` | 502: 1e-12 | `HNR.energy_eps` |
| `src/math.rs` | 509: 2.0, 510: 1e-12 | `TWO`, `HNR.parabolic_flat_eps` |
| `src/math.rs` | 513: 8.0 | `PARABOLIC_PEAK_DENOM` |
| `src/math.rs` | 516: -0.9999, 516: 0.9999 | `HNR.r_clamp` |
| `src/math.rs` | 518: -40.0 | `HNR.db_limit` |
| `src/math.rs` | 520: 10.0, 520: -40.0, 520: 40.0 | `DB_PER_DECADE_POWER`, `HNR.db_limit` |
| `src/math.rs` | 530: 10f32, 530: -48.0, 530: 20.0, 531: 1e-6, 531: 20.0 | `DB_LOG_BASE.powf`, `TIMBRE.relative_floor_db`, `DB_PER_DECADE_AMPLITUDE`, `TIMBRE.amp_eps` |
| `src/math.rs` | 553: 64 | `PERTURB.min_frame_samples` |
| `src/math.rs` | 557: 8.0 | `PERTURB.min_period_samples` |
| `src/math.rs` | 577: 2.0, 578: 1e-12 | `TWO`, `PERTURB.parabolic_flat_eps` |
| `src/math.rs` | 581: 2.0, 581: -0.5, 581: 0.5, 582: 4.0 | `TWO`, `HALF`, `PARABOLIC_HEIGHT_DENOM` |
| `src/math.rs` | 589: 1.5 | `PERTURB.first_search_periods` |
| `src/math.rs` | 593: 0.7, 594: 1.3 | `PERTURB.search_window_lo`, `PERTURB.search_window_hi` |
| `src/math.rs` | 603: 5 | `PERTURB.min_cycles` |
| `src/math.rs` | 607: 2 | `ADJACENT_PAIR` |
| `src/math.rs` | 612: 0.7, 612: 1.3 | `PERTURB.search_window_lo`, `PERTURB.search_window_hi` |
| `src/math.rs` | 618: 2 | `ADJACENT_PAIR` |
| `src/math.rs` | 621: 100.0 | `PERCENT` |
| `src/math.rs` | 626: 5.0 | `PERTURB.max_jitter_pct` |
| `src/math.rs` | 631: 1e-6 | `PERTURB.amp_eps` |
| `src/math.rs` | 635: 2, 636: 20.0 | `ADJACENT_PAIR`, `DB_PER_DECADE_AMPLITUDE` |
| `src/math.rs` | 658: 256 | `CPP.min_frame_samples` |
| `src/math.rs` | 661: 1e-9 | `CPP.silence_energy` |
| `src/math.rs` | 673: 0.5, 673: 0.5, 673: 2.0 | `HANN_A0`, `TAU` |
| `src/math.rs` | 682: 10.0, 682: 1e-12 | `DB_PER_DECADE_POWER`, `CPP.log_floor` |
| `src/math.rs` | 687: 10.0, 687: 1e-12 | `DB_PER_DECADE_POWER`, `CPP.log_floor` |
| `src/math.rs` | 694: 500.0, 695: 60.0, 696: 2 | `CPP.f0_band_hi_hz`, `CPP.f0_band_lo_hz`, `TWO_USIZE` |
| `src/math.rs` | 699: 4 | `CPP.min_band_bins` |
| `src/math.rs` | 704: 2 | `TWO_USIZE` |
| `src/math.rs` | 714: 1e-9 | `CPP.regression_eps` |
| `src/math.rs` | 741: 256 | `CENTROID.min_frame_samples` |
| `src/math.rs` | 744: 1e-9 | `CENTROID.silence_energy` |
| `src/math.rs` | 755: 0.5, 755: 0.5, 755: 2.0 | `HANN_A0`, `TAU` |
| `src/math.rs` | 762: 80.0, 763: 8000.0f32, 763: 2.0, 764: 2 | `CENTROID.band_lo_hz`, `CENTROID.band_hi_hz`, `TWO`, `TWO_USIZE` |
| `src/math.rs` | 776: 1e-9 | `CENTROID.magnitude_eps` |
| `src/math.rs` | 790: 12 | `SEMITONES_PER_OCTAVE_USIZE` |
| `src/math.rs` | 800: 69.0, 800: 12.0, 800: 440.0 | `MIDI_A4`, `SEMITONES_PER_OCTAVE`, `TUNING.a4_hz` |
| `src/math.rs` | 802: 12 | `SEMITONES_PER_OCTAVE_I32` |
| `src/math.rs` | 805: 12, 806: 100.0 | `SEMITONES_PER_OCTAVE_I32`, `CENTS_PER_SEMITONE` |
| `src/math.rs` | 829: 12.0 | `SEMITONES_PER_OCTAVE` |
| `src/math.rs` | 837: 1e-6 | `TIMBRE.amp_eps` |
| `src/math.rs` | 842: 10f32, 842: -48.0, 842: 20.0 | `DB_LOG_BASE.powf`, `TIMBRE.relative_floor_db`, `DB_PER_DECADE_AMPLITUDE` |
| `src/math.rs` | 847: 20.0 | `DB_PER_DECADE_AMPLITUDE` |
| `src/math.rs` | 849: 2 | `TIMBRE.tilt_min_points` |
| `src/math.rs` | 862: 1e-9 | `TIMBRE.tilt_regression_eps` |
| `src/math.rs` | 870: 16 | `TIMBRE.even_odd_partials` |
| `src/math.rs` | 872: 2 | `TWO_USIZE` |
| `src/math.rs` | 878: 10.0 | `DB_PER_DECADE_POWER` |
| `src/math.rs` | 892: 2_800.0, 892: 3_400.0 | `TIMBRE.singers_formant_lo_hz`, `TIMBRE.singers_formant_hi_hz` |
| `src/math.rs` | 896: 1e-12, 896: 100.0 | `TIMBRE.energy_eps`, `PERCENT` |
| `src/math.rs` | 906: 130.0 | `CLASS.bass_max_hz` |
| `src/math.rs` | 908: 175.0 | `CLASS.baritone_max_hz` |
| `src/math.rs` | 910: 220.0 | `CLASS.tenor_max_hz` |
| `src/math.rs` | 912: 290.0 | `CLASS.alto_max_hz` |
| `src/math.rs` | 914: 370.0 | `CLASS.mezzo_max_hz` |
| `src/math.rs` | 927: 1200.0 | `CLASS.dark_max_hz` |
| `src/math.rs` | 929: 2200.0 | `CLASS.warm_max_hz` |
| `src/math.rs` | 931: 3200.0 | `CLASS.balanced_max_hz` |
| `src/math.rs` | 933: 4400.0 | `CLASS.bright_max_hz` |
| `src/math.rs` | 951: -6.0 | `TIMBRE.hollow_even_odd_db` |
| `src/math.rs` | 955: 0.25, 956: 6 | `TIMBRE.strong_partial_rel_amp`, `TIMBRE.rich_min_strong` |
| `src/math.rs` | 958: 2 | `TIMBRE.pure_max_strong` |
| `src/math.rs` | 972: 15.0 | `VOICING.voiced_min_snr_db` |
| `src/math.rs` | 979: 30.0 | `VOICING.identity_min_snr_db` |
| `src/math.rs` | 983: 128 | `NOISE.ring_len` |
| `src/math.rs` | 988: 32 | `NOISE.ring_min` |
| `src/math.rs` | 993: 0.10 | `NOISE.floor_percentile` |
| `src/math.rs` | 1072: 20.0, 1072: 1e-7 | `DB_PER_DECADE_AMPLITUDE`, `NOISE.floor_min_rms` |
| `src/math.rs` | 1081: 100 | `CALIB.frames` |
| `src/math.rs` | 1086: 0.04 | `CALIB.interferer_f0_tol` |
| `src/math.rs` | 1091: 10.0 | `CALIB.interferer_level_margin_db` |
| `src/math.rs` | 1096: 0.3 | `CALIB.voiced_fail_fraction` |
| `src/math.rs` | 1099: 0.15 | `CALIB.interferer_min_fraction` |
| `src/math.rs` | 1102: 0.02 | `CALIB.stable_rel_std` |
| `src/math.rs` | 1172: 2 | `CALIB.too_short_divisor` |
| `src/math.rs` | 1177: 2 | `TWO_USIZE` |
| `src/math.rs` | 1225: 10f32, 1225: 20.0, 1226: 1e-7 | `DB_LOG_BASE.powf`, `DB_PER_DECADE_AMPLITUDE`, `CALIB.interferer_min_rms` |
| `src/math.rs` | 1245: 200.0 | `FORMANT.identity_f0_max_hz` |
| `src/math.rs` | 1252: 350.0 | `FORMANT.display_f0_max_hz` |
| `src/math.rs` | 1260: 0.15 | `FORMANT.harmonic_suspect_frac` |
| `src/math.rs` | 1286: 3 | `N_FORMANTS` |
| `src/math.rs` | 1298: 2.0 | `FORMANT.f2_suspect_harmonic` |
| `src/math.rs` | 1326: 10f32, 1326: -48.0, 1326: 20.0, 1327: 1e-6, 1327: 20.0 | `DB_LOG_BASE.powf`, `TIMBRE.relative_floor_db`, `DB_PER_DECADE_AMPLITUDE`, `TIMBRE.amp_eps` |
| `src/math.rs` | 1336: 6, 1337: 500.0, 1337: 150.0, 1338: 1500.0, 1338: 350.0, 1339: 2600.0, 1339: 400.0, 1340: 1800.0, 1340: 700.0, 1341: -9.0, 1341: 4.0, 1345: 16.6, 1345: 1.4 | `N_SCALAR_FEATURES`, `VOICEPRINT.scalar_means`, `VOICEPRINT.scalar_stds` |
| `src/math.rs` | 1357: 3 | `N_FORMANTS` |
| `src/math.rs` | 1362: 16 | `VOICEPRINT_PROFILE_LEN` |
| `src/math.rs` | 1370: 3 | `N_FORMANTS` |
| `src/math.rs` | 1373: 2 | (moved to data module) |
| `src/math.rs` | 1407: 2 | (moved to data module) |
| `src/math.rs` | 1415: 2 | (moved to data module) |
| `src/math.rs` | 1435: -0.5 | `HALF` |
| `src/math.rs` | 1444: 1e-9, 1444: 1e-9 | `VOICEPRINT.profile_norm_eps` |
| `src/math.rs` | 1448: 0.6, 1448: 0.4 | `VOICEPRINT.scalar_weight`, `VOICEPRINT.timbre_weight` |
| `src/math.rs` | 1453: 100.0, 1453: 100.0 | `PERCENT` |
| `src/metrics.rs` | 12: 3.0, 13: 9.0, 14: 0.25 | `VIB.rate_min_hz`, `VIB.rate_max_hz`, `VIB.rate_step_hz` |
| `src/metrics.rs` | 16: 0.25 | `VIB.edge_margin_hz` |
| `src/metrics.rs` | 18: 0.4 | `VIB.min_explained` |
| `src/metrics.rs` | 20: 8.0 | `VIB.min_extent_cents` |
| `src/metrics.rs` | 41: 2.0, 41: 8, 42: 8 | `VIB.window_secs`, `VIB.min_buffer_samples`, `VIB.min_secs` |
| `src/metrics.rs` | 45: 0.25 | `VIB.max_gap_secs` |
| `src/metrics.rs` | 77: 1200.0 | `CENTS_PER_OCTAVE` |
| `src/metrics.rs` | 82: 1e-4 | `VIB.flat_variance` |
| `src/metrics.rs` | 116: 2.0 | `TWO` |
| `src/metrics.rs` | 124: 1e-9 | `VIB.detrend_eps` |
| `src/metrics.rs` | 137: 0.5, 137: 0.5 | `HANN_A0`, `TAU` |
| `src/metrics.rs` | 142: 1e-6 | `VIB.sweep_end_eps` |
| `src/metrics.rs` | 168: 2.0, 169: 1e-12, 170: 2.0, 170: -0.5, 170: 0.5 | `TWO`, `VIB.parabolic_flat_eps`, `HALF` |
| `src/metrics.rs` | 193: 1e-9 | `VIB.fit_det_eps` |
| `src/fach.rs` | 40: 2000.0, 40: 3600.0, 41: 2300.0, 41: 4500.0 | `FACH.fhe_band_male_hz`, `FACH.fhe_band_female_hz` |
| `src/fach.rs` | 43: 2200.0, 44: 3400.0, 45: 1500.0, 46: 5000.0 | `FACH.cluster_lo_hz`, `FACH.cluster_hi_hz`, `FACH.cluster_valley_from_hz`, `FACH.cluster_width_ceil_hz` |
| `src/fach.rs` | 48: 20, 49: 100.0, 49: 125.0, 49: 160.0, 49: 200.0, 49: 250.0, 49: 315.0, 49: 400.0, 49: 500.0, 49: 630.0, 49: 800.0, 49: 1000.0, 49: 1250.0, 49: 1600.0, 50: 2000.0, 50: 2500.0, 50: 3150.0, 50: 4000.0, 50: 5000.0, 50: 6300.0, 50: 8000.0 | `N_THIRD_OCTAVE_BANDS`, `FACH.third_octave_centers_hz` |
| `src/fach.rs` | 54: 3.0 | `FACH.turnover_hysteresis_db` |
| `src/fach.rs` | 57: 2.0, 58: 4.0, 59: 2.0, 60: 1.5 | `FACH.register_min_jump_st`, `FACH.register_h1h2_delta_db`, `FACH.register_cpp_drop_db`, `FACH.register_jitter_pct` |
| `src/fach.rs` | 80: 256 | `FACH.min_frame_samples` |
| `src/fach.rs` | 83: 1e-12 | `FACH.silence_energy` |
| `src/fach.rs` | 92: 0.5, 92: 0.5, 92: 2.0 | `HANN_A0`, `TAU` |
| `src/fach.rs` | 97: 2 | `TWO_USIZE` |
| `src/fach.rs` | 124: 1e-18 | `FACH.band_energy_eps` |
| `src/fach.rs` | 131: 0.5 | `HALF_F64` |
| `src/fach.rs` | 134: 0.5 | `HALF_F64` |
| `src/fach.rs` | 136: 0.5 | `HALF` |
| `src/fach.rs` | 138: 0.5 | `HALF` |
| `src/fach.rs` | 205: 20 | `N_THIRD_OCTAVE_BANDS` |
| `src/fach.rs` | 207: 20 | `N_THIRD_OCTAVE_BANDS` |
| `src/fach.rs` | 209: 2f32, 209: 6.0, 210: 2f32, 210: 6.0 | `TWO.powf`, `THIRD_OCTAVE_HALF_BAND_EXP` |
| `src/fach.rs` | 218: 10.0, 218: 1e-18 | `DB_PER_DECADE_POWER`, `FACH.ltas_power_floor` |
| `src/fach.rs` | 253: 0.5 | `HALF` |
| `src/fach.rs` | 285: 10.0, 285: 1e-18 | `DB_PER_DECADE_POWER`, `FACH.power_floor` |
| `src/fach.rs` | 302: -170.0 | `FACH.cluster_silent_peak_db` |
| `src/fach.rs` | 312: 3.0 | `FACH.cluster_width_db` |
| `src/fach.rs` | 348: 100.0 | `PERCENT` |
| `src/fach.rs` | 363: 50.0 | `MEDIAN_PERCENTILE` |
| `src/fach.rs` | 373: 10 | `FACH.tessitura_min_frames` |
| `src/fach.rs` | 378: 10.0, 379: 25.0, 380: 50.0, 381: 75.0, 382: 90.0, 383: 2.0, 384: 98.0 | `FACH.tessitura_extreme_lo_pct`, `FACH.tessitura_extreme_hi_pct` |
| `src/fach.rs` | 391: 69.0, 391: 12.0, 391: 440.0 | `MIDI_A4`, `SEMITONES_PER_OCTAVE`, `TUNING.a4_hz` |
| `src/fach.rs` | 396: 12 | `SEMITONES_PER_OCTAVE_USIZE` |
| `src/fach.rs` | 403: 12, 404: 12 | `SEMITONES_PER_OCTAVE_I32` |
| `src/fach.rs` | 462: 1e-6 | `FACH.turnover_flat_eps` |
| `src/fach.rs` | 465: 0.5 | `HALF` |
| `src/fach.rs` | 503: 3 | `FACH.dominant_harmonic_candidates` |
| `src/spectrogram.rs` | 17: 2048 | `SPEC.fft_size` |
| `src/spectrogram.rs` | 20: 512 | `SPEC.hop` |
| `src/spectrogram.rs` | 22: 2 | `TWO_USIZE` |
| `src/spectrogram.rs` | 24: -120.0 | `SPEC.db_floor` |
| `src/spectrogram.rs` | 84: 20.0, 84: 1e-12 | `DB_PER_DECADE_AMPLITUDE`, `SPEC.magnitude_floor` |
| `src/spectrogram.rs` | 99: 0.5, 99: 2.0 | `HANN_A0`, `TAU` |
| `src/synthesis.rs` | 4: 2.0 | (moved to data module) |
| `src/synthesis.rs` | 15: 3 | `N_FORMANTS` |
| `src/synthesis.rs` | 19: 3 | `N_FORMANTS` |
| `src/synthesis.rs` | 31: 1000.0 | `MILLIS_PER_SECOND` |
| `src/synthesis.rs` | 36: 500.0, 37: 50.0, 40: 1500.0, 41: 100.0, 44: 2500.0, 45: 150.0 | `SYNTH.default_formants` |
| `src/synthesis.rs` | 51: 5, 52: 6.0, 53: 150.0 | `SYNTH.default_harmonic_count`, `SYNTH.default_delta_f_hz`, `SYNTH.default_f0_hz` |
| `src/synthesis.rs` | 55: 150.0 | `SYNTH.default_f0_hz` |
| `src/synthesis.rs` | 81: 20.0 | `SYNTH.min_target_f0_hz` |
| `src/synthesis.rs` | 93: 3 | `N_FORMANTS` |
| `src/synthesis.rs` | 100: 2, 100: 2, 101: 10.0 | `SQUARED`, `SYNTH.resonance_scale` |
| `src/synthesis.rs` | 105: 100.0, 106: 0.1 | `SYNTH.rolloff_corner_hz`, `SYNTH.base_gain` |
| `src/synthesis.rs` | 137: 0.0001 | `SYNTH.amp_cutoff` |
| `src/synthesis.rs` | 168: 0.5, 168: 0.5 | `SYNTH.headroom` |
| `src/frame.rs` | 21: 2048 | `STREAM.frame_samples` |
| `src/frame.rs` | 26: 3, 28: 500.0, 29: 80.0, 32: 1500.0, 33: 120.0, 36: 2500.0, 37: 160.0 | `N_FORMANTS`, `FORMANT.default_formants` |
| `src/frame.rs` | 44: 0.4, 45: 50.0, 46: 1000.0 | `VOICING.min_confidence`, `VOICING.f0_min_hz`, `VOICING.f0_max_hz` |
| `src/frame.rs` | 63: 3 | `N_FORMANTS` |
| `src/frame.rs` | 138: 11_025.0 | `LPC.decimation_target_hz` |
| `src/frame.rs` | 140: 2, 140: 1000.0, 140: 8, 140: 20 | `LPC.order_base`, `LPC.order_hz_per_pole`, `LPC.order_min`, `LPC.order_max` |
| `src/frame.rs` | 143: 0.97 | `LPC.preemphasis` |
| `src/spatial.rs` | 54: 2048 | `SPATIAL.fft_n` |
| `src/spatial.rs` | 56: 1024 | `SPATIAL.hop` |
| `src/spatial.rs` | 58: 512 | `SPATIAL.calib_frames` |
| `src/spatial.rs` | 62: 200.0, 63: 8000.0 | `SPATIAL.band_lo_hz`, `SPATIAL.band_hi_hz` |
| `src/spatial.rs` | 66: -65.0 | `SPATIAL.quiet_dbfs` |
| `src/spatial.rs` | 68: 0.1 | `SPATIAL.score_ema_alpha` |
| `src/spatial.rs` | 71: 0.5 | `SPATIAL.covered_coherence` |
| `src/spatial.rs` | 78: 0.2 | `SPATIAL.rank_warn` |
| `src/spatial.rs` | 82: 3 | `SPATIAL.calib_max_wall_factor` |
| `src/spatial.rs` | 84: 0.85, 85: 0.40 | `SPATIAL.score_tv`, `SPATIAL.score_not_tv` |
| `src/spatial.rs` | 88: 0.10 | `SPATIAL.single_weak_coverage` |
| `src/spatial.rs` | 90: 0.2 | `SPATIAL.sep_min` |
| `src/spatial.rs` | 93: 0.05 | `SPATIAL.disc_min` |
| `src/spatial.rs` | 95: 0.3, 96: -0.3 | `SPATIAL.contrast_tv`, `SPATIAL.contrast_user` |
| `src/spatial.rs` | 119: 2 | `CHANNELS` |
| `src/spatial.rs` | 185: 2 | `TWO_USIZE` |
| `src/spatial.rs` | 190: 0.5, 190: 0.5, 190: 2.0 | `HANN_A0`, `TAU` |
| `src/spatial.rs` | 198: 2, 199: 2 | `SPATIAL.accumulator_frames` |
| `src/spatial.rs` | 287: 10.0, 287: 2, 287: 1e-18 | `DB_PER_DECADE_POWER`, `CHANNELS`, `SPATIAL.energy_floor` |
| `src/spatial.rs` | 348: 1e-3 | `SPATIAL.weight_eps` |
| `src/spatial.rs` | 358: 1e-4 | `SPATIAL.joint_weight_eps` |
| `src/spatial.rs` | 368: 1e-12 | `SPATIAL.denominator_eps` |
| `src/spatial.rs` | 378: 1e-12 | `SPATIAL.denominator_eps` |
| `src/spatial.rs` | 437: 1e-30, 438: 4.0, 439: 0.5, 440: 0.5 | `SPATIAL.eigen_eps`, `EIGEN2_DISCRIMINANT_FACTOR`, `HALF_F64` |
| `src/spatial.rs` | 443: 1e-30 | `SPATIAL.eigen_eps` |
| `src/spatial.rs` | 450: 1e-30 | `SPATIAL.eigen_eps` |
| `src/spatial.rs` | 461: 1e-30 | `SPATIAL.eigen_eps` |
| `src/spatial.rs` | 523: 1e-12 | `SPATIAL.denominator_eps` |
| `src/analysis.rs` | 7: 2048 | `STREAM.frame_samples` |
| `src/analysis.rs` | 16: 1024 | `STREAM.gpu_diff_len` |
| `src/analysis.rs` | 20: 3, 22: 500.0, 23: 80.0, 26: 1500.0, 27: 120.0, 30: 2500.0, 31: 160.0 | `N_FORMANTS`, `FORMANT.default_formants` |
| `src/analysis.rs` | 201: 64 | `STREAM.gpu_diff_workgroup` |
| `src/analysis.rs` | 257: 3 | `N_FORMANTS` |
| `src/analysis.rs` | 337: 0.4, 337: 50.0, 337: 1000.0 | `VOICING.min_confidence`, `VOICING.f0_min_hz`, `VOICING.f0_max_hz` |
| `src/analysis.rs` | 401: 11_025.0 | `LPC.decimation_target_hz` |
| `src/analysis.rs` | 403: 2, 403: 1000.0, 403: 8, 403: 20 | `LPC.order_base`, `LPC.order_hz_per_pole`, `LPC.order_min`, `LPC.order_max` |
| `src/analysis.rs` | 406: 0.97 | `LPC.preemphasis` |
| `src/tract.rs` | 17: 5, 18: -5.10, 18: 0.88, 19: 0.66, 19: 2.22, 20: 3.86, 20: 1.35, 21: 0.00, 21: -2.69, 22: -3.48, 22: -1.70 | `TRACT.speed_of_sound_cm_s`, `TRACT.sweep_lo_hz`, `TRACT.sweep_hi_hz`, `TRACT.sweep_step_hz`, `TRACT.bisect_iters`, `N_RESONANCES`, `TRACT.n_resonances`, `TRACT.min_diameter_cm` |
| `src/tract.rs` | 28: -5.6, 29: 4.4, 30: -3.1, 31: 2.7 | `TRACT.q1_min`, `TRACT.q1_max`, `TRACT.q2_min`, `TRACT.q2_max` |
| `src/tract.rs` | 35: 41 | `TRACT.grid_n` |
| `src/tract.rs` | 42: 4.0 | `TRACT.invert_max_dist` |
| `src/tract.rs` | 47: 150.0, 48: 300.0 | `TRACT.invert_f1_scale_hz`, `TRACT.invert_f2_scale_hz` |
| `src/tract.rs` | 53: 8.0, 54: 22.0 | `TRACT.vtl_min_cm`, `TRACT.vtl_max_cm` |
| `src/tract.rs` | 80: 3 | `N_RESONANCES` |
| `src/tract.rs` | 92: 2.0 | `TAU` |
| `src/tract.rs` | 108: 3 | `N_RESONANCES` |
| `src/tract.rs` | 113: 3 | `N_RESONANCES` |
| `src/tract.rs` | 120: 0.5 | `HALF` |
| `src/tract.rs` | 129: 0.5 | `HALF` |
| `src/tract.rs` | 136: 3 | `N_RESONANCES` |
| `src/tract.rs` | 160: 2, 160: 2 | `TRACT.grid_min_n` |
| `src/tract.rs` | 225: 1e-3 | `TRACT.idw_eps` |
| `src/tract.rs` | 249: 0.3, 250: 0.7, 251: 3.0, 251: 4.0, 252: 5.0, 252: 4.0 | `TRACT.vtl_weight_f2`, `TRACT.vtl_weight_f3`, `QUARTER_WAVE_ODD_MULTIPLES`, `QUARTER_WAVE_DENOM` |
| `src/tract.rs` | 263: 15.53, 263: 17.6, 263: 2.0 | `ADULT_FEMALE.vtl_cm`, `ADULT_MALE.vtl_cm`, `TWO` |
| `src/tract_data.rs` | 36: 35_000.0, 41: 100.0, 42: 4_000.0, 46: 20.0, 48: 30, 54: 0.05 | (moved to data module) |
| `src/audio_file.rs` | 185: 0.5 | `HANN_A0`, `TWO`, `RESAMPLE.same_rate_tol_hz` |
| `src/audio_file.rs` | 190: 0.45, 191: 32 | `RESAMPLE.cutoff_of_rate`, `RESAMPLE.half_taps` |
| `src/audio_file.rs` | 204: 2.0, 205: 1e-6 | `TWO`, `RESAMPLE.sinc_center_eps` |
| `src/audio_file.rs` | 210: 0.5, 210: 0.5, 211: 2.0 | `HANN_A0`, `TWO` |
| `src/audio.rs` | 35: 20.0 | `DEFAULT.glide_ms` |
