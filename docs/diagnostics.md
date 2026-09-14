# Engineering Console (diagnostics)

VoxLabs carries the diagnostics page of Vocal Tract Lab 0.10.0
(`org.vocaltract.lab3d`, `AdminConsoleActivity` + `DiagnosticsRuntime` +
`DiagnosticStore` + `DiagnosticsCore`), ported from the shipped APK into
Rust/egui. Open it with the **DIAGNOSTICS** chip at the top of every screen;
**Return to VoxLabs** goes back where you were.

The page exists so a phone with no `adb` can report on itself: what the
app knows about its microphone, its DSP timing, its pitch/formant/tract
state, the events it logged, and nine self-tests — and can put all of it
in a JSON bundle in `Downloads/VoxLabs` and hand it to the share sheet.

Privacy stance, unchanged from the source: **derived metrics only, no raw
audio, no upload, no automatic model mutation.** The app has no Internet
permission.

## What is on the page

In order, as in the source app:

| Section | Content |
|---|---|
| **Engineering Console** | `Session <id>` + the privacy line. |
| **Active model** | The compiled-in mode (`vox-core live_model 0.1.0`), its stages, frame/hop, and the source locks (FNV-1a of `pipelines/live_model.toml` and `pipeline.toml`). The source app's "Active 3D atlas". |
| **Offline diagnostic assistant** | Provider, health score /100, summary, the `Latest:` line, and every finding as `SEVERITY: title / Evidence / Action`. Buttons **Refresh advice**, **Run self-test**, **Enable/Disable live overlay**. On Android, when live mode lacks the microphone permission, an extra **Open app settings** button (VoxLabs addition). |
| **Room calibration** | `Completed N derived-only steps • M frames • no raw audio • model unchanged` after a Room-screen calibration. The source app's "Singer calibration". |
| **Self-test results** | `PASS  code: message` / `CHECK  code: message`. |
| **Propose a refinement** | Expected F0, vowel, R1–R6, notes, the training checkbox, **Save proposed correction**. Same validation and messages as the source. |
| **Progress and review bundle** | **Export and share AI review bundle**. |
| **Recent session events** | Newest 30 events, newest first, `HH:MM:SS SEVERITY category/code: message`. **Refresh log**, **Clear local logs** (with the confirmation dialog), **Return to VoxLabs**. |

## The offline assistant

`src/diagnostics/core.rs::assess` is the source app's rule set verbatim:
the same 14 findings (codes, titles, evidence strings, recommended actions,
confidences), sorted by severity then confidence, health = 100 − 25 per
ERROR − 10 per WARNING − 2 per INFO, and the same three summaries. Every
threshold is in `pipeline.toml` under `[diagnostics]` (`DiagnosticsConfig`),
so the drift test covers it.

## How VoxLabs fills the metrics

The source app's `DiagnosticMetrics` fields, and what feeds them here
(`src/ui/diagnostics.rs::derived_metrics`, once per new analysis frame):

| Field | VoxLabs source |
|---|---|
| `source` | `idle` (no audio stream), `live`, or `file` during an import |
| `processing_ms`, `frame_budget_ms` | runner `last_hop_us`, `hop_budget_us` (Android); desktop has no runner: 0 and the frame length |
| `dropped_frames` | ring-buffer xruns + dropped tap messages |
| `voiced`, `f0_hz` | `VocalProfile::valid`, `f0` |
| `f0_confidence`, `raw_f0_hz` | the `yin` tap's `F0Track` (Android); desktop: 1.0 when voiced, `f0` |
| `pitch_decision`, `pitch_rejected` | `accepted` / `gated_noise` (`voiced_but_noisy`) / `gated` / `unvoiced` |
| `harmonicity` | HNR, dB |
| `snr_db` | frame SNR over the learned floor |
| `noise_state`, `noise_confidence`, `noise_floor_db` | room-calibration state; ambient floor in dBFS once calibrated |
| `tract_confidence`, `posterior_abstained`, `abstention_reason` | formant reliability grade: Identity 1.0, DisplayOnly 0.6, Reject 0.1 + `formants_suspect`; unvoiced 0.0 + `unvoiced`/`held` |
| `formant_candidates_hz` | F1..F3 |
| `tract_params` (bundle) | Story (q1, q2), VTL estimate, basis, live/held — replaces the source's `articulators` |
| `renderer_mode` | `glow_gles` (Android) / `wgpu` (desktop) |
| `microphone_granted` | `RECORD_AUDIO` grant (Android); always true on desktop |
| `synthesizer_active` | the cpal output (monitor) stream is open |
| `mean_formant_std_hz`, `relative_area_std`, `background_changed`, `noise_bands_db` | not measured in VoxLabs; 0 / false / empty, so their rules never fire |

Silence therefore reads as `very_low_tract_confidence` (ERROR) +
`posterior_abstained` (WARNING), health 65 — exactly what the rules say
about a frame with no voice in it. Sing a vowel and the score climbs.

## Events

`src/diagnostics/runtime.rs::log` writes `timestamp, session, category,
code, severity, message, evidence{}` lines to
`<files>/diagnostics/session-<id>.jsonl` (4 MiB cap; 20 newest session
files kept). Categories carried over: `lifecycle`, `runtime`
(`derived_state_sample` every 500 ms or on a drop-count change), `audio`,
`renderer`, `calibration`, `self_test`, `refinement`, `export`, `privacy`,
`ui`, `crash` (a panic hook; the marker is reported as
`recovery/previous_crash_recovered` on the next launch). VoxLabs adds
`log/<module>`: every `log::warn!`/`error!` the app emits (the audio
engine failing to open, for one) becomes an event, so the console shows
what logcat would.

Times in the event list are UTC (`std` has no local time zone).

## Self-tests

| Code | Check |
|---|---|
| `pipeline_definition` | `live_model.toml` loads; frame == `ANALYSIS_FRAME` |
| `inversion_grids` | both 41×41 grids build; the neutral shape inverts to ~(0, 0) |
| `config_contract` | `pipeline.toml` parses and equals the code defaults |
| `synthesis_math` | 512 oscillator-bank samples, finite and not silent |
| `audio_input_format` | cpal's default input device and format (fails with no device or no permission) |
| `private_storage` | the data folder accepts a write |
| `pitch_estimator` | YIN reads a 110 Hz fixture within 105–116 Hz |
| `noise_floor` | a quiet fixture learns; a loud frame reads ≥ 30 dB SNR |
| `microphone_permission` | `RECORD_AUDIO` granted |

## Export bundle

`voxlabs-diagnostics-<session>.json`, written through `MediaStore.Downloads`
to `Downloads/VoxLabs` and offered to the share sheet (desktop:
`~/Downloads/VoxLabs/`). Layout is the source app's
`vocaltract3d.diagnostic-bundle/1.1` with `schema_version`
`voxlabs.diagnostic-bundle/1.1`: privacy, app, device, `latest_metrics`,
`assistant_assessment`, `self_tests`, `singer_calibration`, `shared_model`
(null), `events` (last 1500), `refinements` (last 500).

## Files

- `src/config/diagnostics.rs` — `DiagnosticsConfig` (`[diagnostics]`).
- `src/diagnostics/{core,store,runtime,self_test,export,ids}.rs`.
- `src/diag.rs` — the `log` tee.
- `src/ui/diagnostics.rs` — the screen, the metrics feed, the overlay line.

## Deviations from the source page

- Section titles "Active 3D atlas" → "Active model", "Singer calibration"
  → "Room calibration"; "Return to vocal tract" → "Return to VoxLabs";
  Downloads folder `VocalTractLab` → `VoxLabs`; schema tags renamed.
- The **Open app settings** button under the assistant (Android, only
  while the permission is missing).
- `articulators` in the bundle is null; `tract_params` carries VoxLabs'
  articulatory state instead.
- Event times are UTC.
