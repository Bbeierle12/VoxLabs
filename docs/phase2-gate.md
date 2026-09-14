# Phase 2 gate — the rest of C2 wrapped; provenance; validation

Plan v3 §5 Phase 2. What was built, how it is proven on the host, and
what remains for the phone.

## What runs now

`pipelines/live_model.toml` is nine stages, every kernel wrapped, each
with the kernel's behaviour as its contract test:

| Stage | Backend | Out | Kernel |
|---|---|---|---|
| yin | `yin_cpu` | F0Track | `math::yin_pitch` + the confidence/range gate |
| voicing | `snr_hum` | F0Track (gated, with SNR) | `NoiseFloor`, `interferer_match` — `frame::FrameAnalyzer`'s gates |
| lpc | `lpc_levinson` | FormantTrack | decimate → LPC → Aberth roots |
| harmonics | `goertzel` | HarmonicSeries | `harmonic_amplitudes` |
| metrics | `classic` | VoiceMetrics | HNR, H1–H2, jitter/shimmer, CPP, centroid |
| contour | `f0_contour` | VoiceMetrics (+vibrato, steadiness) | `metrics::F0Contour` at hop cadence |
| inverse | `grid_story` | TractParams | the 41×41 grid |
| tract | `story_two_mode` | AreaFunction | the Story area function |
| stft | `rustfft` | Spectrum | the scrolling spectrogram's FFT (D14: hop 1024) |

`android.rs::LegacyTail` is gone. The UI's profile, waterfall and scope
come from the wires (`pipeline::consumers::profile_from_wires`); the
room-calibration pass and the raw-capture export stay at frame cadence
(every second hop) so their frame counts and file layout are unchanged.
YIN and LPC run once per hop.

Two more mode files: `pipelines/fingerprint.toml` (yin → voicing → lpc →
harmonics → metrics → contour) and `pipelines/calibrate.toml` (yin →
voicing → stft). `PipelineDefinition::by_name_or_path` loads a compiled-in
mode by name or any `.toml` path.

## Provenance (D11)

`pipeline::provenance::ProvenanceRecord`: the definition as loaded, its
SHA-256, the built format, every stage's name/backend/version and wire
signature, the SHA-256 of the effective parameters, the build (crate,
version, OS, arch, profile, `pipeline.toml` digest) and the input (live,
or a file with its digest and sample count). The runner writes one when a
pipeline is built (`<files>/diagnostics/provenance-<mode>-<ms>.json`, plus
a `provenance/pipeline_built` event); the harness writes one per file.
The Engineering Console's bundle carries the latest record and the
Evidence output.

SHA-256 is `hash.rs`, dependency-free, tested against the FIPS vectors.

## Harness and validation

```
voxlab run <mode|mode.toml> <audio|dir> [--out DIR] [--sr HZ]
voxlab validate <results_dir|x.provenance.json>
```

`run` records `<stem>.taps.jsonl` (every tapped wire, every hop) and
`<stem>.provenance.json`. `validate` (crate `vox-validation`) reads the
record, rebuilds the pipeline from the embedded definition, checks the
input's digest and the parameter digest, re-runs the input, and compares
every tap within the `[tolerance]` bands (`pipeline::compare`); it writes
`<stem>.validation.json` and `<stem>.evidence.json`.

The Evidence output is the Vocal Tract Lab Evidence-tab layout: a NOT
VALIDATED banner, baseline acceptance gates with status (contract checks,
device self-test, provenance recorded, provenance round-trip, hop
deadline, room calibration, and the three that are honestly "not
established"), and a statement.

## Gate, host side

| Check | Where | Result |
|---|---|---|
| Whole chain equals `FrameAnalyzer` field for field (f0, formants, harmonics, every metric, vibrato, steadiness) at frame cadence | `pipeline::consumers::tests::pipeline_profile_equals_frame_analyzer_profile` | pass |
| Each stage equals its kernel | `pipeline::stages::*::tests` | pass |
| Fingerprint mode reproduces enrollment features on a synthetic fixture: f0 and fresh formants within the bands, voicing agreement ≥ 95 %, enrolled voiceprint match ≥ 99 | `vox-validation/tests/round_trip.rs::fingerprint_mode_reproduces_the_analyzers_enrollment_features` | pass |
| Provenance round-trips: record → rebuild → re-run → every tap within band; a changed input is caught by the digest | `…::a_recorded_run_round_trips_within_the_bands` | pass |
| Every compiled-in mode loads and builds | `pipeline::definition::mode_tests` | pass |
| Contract checks on device | Engineering Console → Run self-test (`contract.*` lines) | to be read off the phone |
| Phase 1 timing with the full chain | `runtime/pipeline_report` events in the bundle | to be read off the phone |

`cargo test --workspace`: 188 library tests, 6 harness tests, 2
validation tests. `cargo clippy --workspace --all-targets -- -D warnings`
clean. Android and wasm library type-checks pass.

## Not done in this phase

- `loudness` (ISO 532-1): no reference vectors are available, so no
  contract test is possible; deferred (DECISIONS.md, completion-plan call 3).
- The `SpatialCal` two-microphone calibration stays a native Android
  capture, not a single-channel stage.
- The desktop GPU engine (`analysis.rs`) still runs `FrameAnalyzer` live
  on desktop; D15 retires it in Phase 5c.
