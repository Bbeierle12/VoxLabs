# Phase 5 gate — the phone shell (5a), the desktop engine (5c), wasm (5b)

Plan v3 §5 Phase 5 (`docs/PLAN_v3_completion.md` §6–8), executed under
the Pixel-only directive (D18 amendment). One decision shapes the phase
and is recorded in `DECISIONS.md`: **the phone's shell is the existing
egui NativeActivity build, not Tauri.**

## 5a — the phone shell

What the plan asked for, and what this delivers on the egui shell:

| Plan §6 item | Delivered |
|---|---|
| Live Model on the Pixel with the Phase 1 numbers | Unchanged: the runner, the taps and the Console's `runtime/pipeline_report` evidence |
| The core under the shell; the shell shell-agnostic (D9) | `src/shell/`: `engine::LiveEngine` owns the runner on the live ring buffer, starts a mode by name, **switches modes at runtime** (the worker hands its ring-buffer consumer and observer back on `Runner::stop`), retires on relaunch; `consumers::WireConsumers` is the one hop observer both targets use. The phone and the desktop run the same code path. |
| Mode selection | The Room screen's PIPELINE card has a mode row (`live_model`, `fingerprint`, `calibrate`, `atlas`, `choir`); a tap switches the live runner without reopening the microphone; the choice persists in `mode.txt` and comes back on relaunch. |
| Coral's views as tap renderers | `ui::tap_views::rehearsal_card` (four S/A/T/B cards, chord, consonance, drift, pairs) from the `multi_f0` + `satb` taps through `choir::pipeline::ChoirHarmony`; the waterfall reads any mode's `Spectrum` tap (max-pooled to its bin count). |
| The atlas mesh | `ui::tap_views::mesh_card`: centerline, every ring and the uncertainty envelope from the `mesh` tap, projected onto the renderer's own x–y plane (the anatomical frame is unknown, and said so on the card). |
| Experiment 2 (WebGL2 vs WebGPU in the WebView) | **Not run**: it measures a WebView the shell does not use. Recorded, not hidden. |
| Native-vs-desktop tolerance bands (D5) | The procedure exists end to end; the numbers need the phone (below). |
| The TypeScript worker deleted; Coral capture → cpal | The Rust taps now have renderers on the phone, which was 5a's condition; the worker in `apps/coral/src/audio/` is superseded by `src/choir/` and can be deleted once Brandon confirms nothing else in the imported app is wanted (the imported app itself does not run on the phone). Capture is cpal on both targets already. |

### D5 procedure (arm64 vs x86_64)

1. On the Pixel: Engineering Console → **Record fixture taps** (every mode
   over its compiled-in fixture: the contract vowel, or the SATB chord for
   `choir`, 1 s at 48 kHz) → **Export fixture taps** (to Downloads/VoxLabs).
2. On the host: `voxlab fixture-taps <dir>` writes the same files.
3. `voxlab compare-taps host/fixture-<mode>.taps.jsonl phone/fixture-<mode>.taps.jsonl`
   prints every wire type's worst delta against the current `[tolerance]`
   bands and writes `<stem>.compare.json`; the bands are then set from
   those numbers.

Until the phone files come back the bands stay at the plan's starting
values. `pipeline::record` round-trips a file with itself inside the
bands on the host (`record::tests`).

## 5c — D15 on the desktop

Done, reduced as the D18 amendment reduced it:

- `analysis::AnalysisEngine`, `GpuYin`, `yin_diff.wgsl`, `yin_scan.wgsl`,
  `math::yin_f0_from_diff_cumsum` and the `wgpu` / `pollster` analysis
  dependencies are deleted. `wgpu` remains only as eframe's desktop
  renderer feature.
- The desktop entry (`lib.rs::run`) starts `shell::engine::LiveEngine`
  on the live stream exactly as the phone does; the egui dashboard
  attaches the runner's taps; the mode selector works on the desktop.
- Gate as re-stated in completion-plan call 5 ("the desktop CPU engine
  equals the `voxlab` harness bit-for-bit and `gpu_yin_matches_cpu`'s
  CPU side"): the desktop engine *is* the harness's pipeline now — the
  same stages from the same mode files through the same `Framer` — so
  equality holds by construction; the YIN stage's contract
  (`yin_matches_direct`) is the CPU side the deleted GPU test compared
  against. The frozen GPU baseline (plan §7.1) could not be produced
  here (no adapter) and is not needed for this gate.
- egui stays (O3 answered for VoxLabs: the shell is egui on both
  targets; Resonator is not affected by this branch).

## 5b — browser wasm

Dropped from the schedule by the D18 amendment. The library still
type-checks for `wasm32-unknown-unknown` (CI), which is all the
directive keeps.

## Gate

| Check | Result |
|---|---|
| `shell::engine` starts, switches and retires a runner on one stream | `engine::tests::switching_modes_reuses_the_parked_stream` pass |
| Fixture taps record and compare | `record::tests` pass |
| Desktop builds and runs without a GPU compute path | `cargo check` on the desktop target; no `wgpu` symbol outside eframe |
| Android and wasm type-check | pass |
| On the Pixel | mode switch live_model → choir → atlas from the Room screen; the rehearsal card sings; the mesh card draws; fixture taps exported — **to be read off the phone** |

`cargo test --workspace`: 228 library, 6 harness, 4 validation tests.
