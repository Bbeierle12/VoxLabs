# Plan v3 — completion plan for the remaining phases

*2026-09-14. Written against `docs/PLAN_v3_pipeline.md` (the plan), `DECISIONS.md`, `docs/STATUS.md` and the code on branch `claude/epic-lamport-euju2k` (PR #4). It does not change the plan's decisions; it says, for each remaining phase, what is already in hand, what the work actually is in this codebase, what blocks it, how it is proven, and what needs Brandon's call. Estimates are working days for one developer with Claude Code and are marked as estimates.*

---

## 0. Where the branch stands

| Phase | State | Evidence |
|---|---|---|
| 0 Housekeeping | **Done** | crate `vox-core`, `src/ui/` split, `DECISIONS.md`, every DSP literal in `pipeline.toml` (drift test), `docs/phase0-extraction.md` |
| 1 Walking skeleton | **Host side done; device gate not run** | `Stage`, builder with adapter-chain errors, preallocating `Runner`, `Tap`, `live_model.toml`, four wrapped stages with contract tests, Android loop through the runner, Room PIPELINE card; 174 host tests. `docs/phase1-gate.md` is the procedure; no numbers from the Pixel yet |
| Field blocker | **Open** | Live microphone input on the Pixel does not detect voice. Two builds shipped for it: the runtime `RECORD_AUDIO` request (14d790c) and the Engineering Console (46fd953) whose self-tests and event log are the diagnosis tool. Nothing further can be measured on the phone until this is closed |
| 2–8 | Not started | — |

Two facts changed since the plan was written and bear on the schedule:

1. **O4 is half-resolved.** The shipped Vocal Tract Lab 0.10.0 APK decompiles cleanly (jadx). The inverse (`TractFit`, `TractPosterior`, `TemporalAtlasFilter`, `PitchContinuityGate`, `StationaryNoiseTracker`), the reduced model loader (`ReducedModelAsset`) and its data (`reduced_model.json`: `vt3d-frozen-mri-pca-v0.7.0`, 4 modes × 32 sections, 16 kHz frame 1024 hop 512, a ridge-regularized formant→mode Jacobian, uncertainty block; `tract_lumen_v2.bin`: the surface mesh) are readable. What is missing without the repository: comments, unit tests, replay fixtures, and the Kotlin originals. Phase 3 can start from the decompiled Java; its parity gate still wants the source.
2. **Part of Phase 2's provenance/validation layer exists in another shape.** The Engineering Console's store, bundle and event log (`src/diagnostics/`) are the file, JSON and export plumbing D11 needs; the provenance record is a new document type on top of them, not new infrastructure. The Evidence-tab format (D11's output) is also now in hand from the APK's web assets: a "NOT VALIDATED" banner, a `Baseline acceptance gates` list of *gate → status* rows, and a plain-language "what this is" paragraph.

Naming, once, to avoid a confusion that runs through the plan: **"Vocal Tract Lab"** is Brandon's Kotlin app (D10, O4, Phase 3 — the source of the posterior inverse and the MRI-reduced model). **"VTL" / VocalTractLab** is Birkholz's GPL C++ library (D3, O1, Phase 6). Both appear below; they are different things.

---

## 1. Order of work

The plan's order stands: **1-close-out → 2 → 3 → 4 → 5a → 5c → 5b → 6 → 7 → (8)**, with two adjustments the code argues for:

- The STFT/`Spectrum` stage is pulled forward from Phase 5c ("re-home spectrogram") into Phase 2, because Phase 4's Coral port consumes `Spectrum` and Phase 2 already touches every per-frame consumer. 5c keeps the desktop re-homing.
- The Cargo workspace split (D2) happens at the start of Phase 2, not at the Coral import. `vox-harness` and `vox-validation` are Phase 2 deliverables and are cleaner as crates than as `src/bin/` targets; doing it before Coral means the subtree lands into a workspace rather than forcing one mid-import.

Phases 6, 7 and 8 are unchanged in scope and stay after 5c; 5b (browser wasm) remains deferrable and blocks nothing.

Dependency picture:

```
1-close-out ─┬─► 2 (wrap C2, provenance, harness, fingerprint/calibrate)
             │      ├─► 3 (VTL-app backends; needs O4 source for full parity, O1 for shipping)
             │      ├─► 4 (Coral subtree, choir stages, Tauri/React shell)
             │      │      ├─► 5a (Android under Tauri)  ─► 5c (desktop under Tauri, D15)
             │      │      │                                   └─► 6 (Workbench + VTL GPL backend) ─► 7 (Anatomy)
             │      │      └─► 5b (wasm; optional, independent)
             └─ Fach Lab M1–M5 runs alongside on the harness throughout
```

---

## 2. Phase 1 close-out (est. 2–5 days; the spread is the microphone bug)

**Goal.** The gate measured on the Pixel, and the phone able to report on itself without adb.

1. **Close the live-audio fault.** Read the console screenshot or bundle: `audio_input_format` and `microphone_permission` self-tests, the `log/*` events, health score. Fix whatever it names (candidates already ranked: permission not held; AAudio stream refused; input rate mismatch; stream open but samples not reaching the ring). One commit, one APK.
2. **Put the runner's periodic report in the event log.** The 11-second `pipeline … worst hop … misses …` line is `log::info!`, which the tee does not capture. Emit it as a `runtime/pipeline_report` event with the numbers as evidence, so the ten-minute run's numbers are in the export bundle and the gate can be read without logcat. Small change in `runner.rs` + `diag.rs`.
3. **Run the gate** per `docs/phase1-gate.md`: ten minutes, Room screen up, export bundle. Record per-stage timing, worst hop vs the 21.3 ms budget, misses, mic-to-render latency in `docs/phase1-gate.md` as a results section. Pass criteria are already in `live_model.toml`.
4. **Contract tests on device.** The honest column today is "builds; not executed on device". Cheapest route: `cargo test --no-run --target aarch64-linux-android`, push the test binary with adb, run it in `/data/local/tmp`. This needs adb on a desktop (platform-tools is a small download; it is also what Phase 5a's Tauri tooling needs). If adb stays off the table, the fallback is an in-app "Run contract tests" self-test that links the contract-test bodies into the console — more code, but no cable.
5. Rename the launcher label to "VoxLabs" (a leftover from Phase 0; one line in `Cargo.toml`).

**Exit.** `docs/phase1-gate.md` carries measured numbers; `DECISIONS.md` D18 note says Phase 1 gate passed/failed with the numbers; the field blocker is closed or its cause is written down.

**Decision needed.** Install adb on the desktop (recommended), or build the in-app contract-test runner.

---

## 3. Phase 2 — Wrap the rest of C2; provenance; validation (est. 15–20 days)

**Goal (plan).** Fingerprint mode reproduces enrollment/match on the `voxlab` synthetic fixtures within tolerance; provenance round-trips.

**Completion criterion the plan implies but does not state:** `android.rs::LegacyTail` is deleted. Everything it runs today (SNR/hum gates, harmonics, voice metrics, spectrogram, capture export, calibration) is a stage or a tap consumer, and YIN/LPC run once per hop, not twice.

### 3.1 Workspace split (2 days)
Convert to a Cargo workspace: `vox-core` (the library, unchanged paths), `vox-harness` (today's `src/bin/voxlab.rs`), `vox-validation` (new, small). Keep the Android cdylib and the desktop binary in `vox-core` so `cargo apk` and `build-dev-apk.sh` do not change. No new dependencies.

### 3.2 Stages (7–9 days)
Wrap, in this order, each with its existing tests as contract tests and no kernel changes:

| Stage | Backend id | In → Out | Kernel today |
|---|---|---|---|
| `voicing` | `snr_hum` | `AudioFrame` + `F0Track` → `F0Track` (gated) | `frame.rs` gate logic + `math::NoiseFloor`, `interferer_match` |
| `stft` | `rustfft` | `AudioFrame` → `Spectrum` (hop 1024, `overlap` param; D14) | `spectrogram.rs` (moves from hop 512) |
| `harmonics` | `goertzel` | `AudioFrame` + `F0Track` → `HarmonicSeries` | `math::harmonic_amplitudes` |
| `metrics` | `classic` | `AudioFrame` + `F0Track` + `HarmonicSeries` → `VoiceMetrics` | `hnr_db`, `h1_h2_db`, `cpp_db`, `cycle_perturbation`, `spectral_centroid`, `spectral_tilt_db_per_octave` |
| `contour` | `f0_contour` | `F0Track` → `VoiceMetrics` (vibrato, steadiness) | `metrics::F0Contour` (stateful; allocation at init) |
| `loudness` | `iso532_1` | `AudioFrame` → `Loudness` | **new code**; contract vectors from the Python reference (see decision) |

`lpc` moves behind `voicing` so formants run on voiced frames only, as today. The type table gains nothing: `VoiceMetrics` already lists HNR, CPP, jitter, shimmer, centroid, tilt, LTAS.

Session-level measures stay **tap consumers**, not stages: voiceprint (`math::build_voiceprint` over a capture), LTAS, cluster stats, tessitura, turnover/register events (`fach.rs`). They accumulate across hops, which the per-hop contract forbids, and they already run in the harness. They become a `vox-core::consumers` module the shell and the harness both call.

### 3.3 Provenance (D11) and `vox-validation` (3 days)
- `ProvenanceRecord`: pipeline name + TOML hash, ordered stage idents (name, semver, backend), `PipelineParams` hash + the mode file's `[params]` overrides, target triple, profile, feature flags, `rustc` version, input hash (sha256 of the fixture, or "live"). Written by the `Runner` at build (into the diagnostics store as an event and a file) and by the harness per run.
- Round-trip: `vox-validation` rebuilds the pipeline from a record and re-runs the fixture; every tap within the D5 band. This is the gate's second half and is a test.
- Evidence output: a `gates` list of *gate → status* rows plus the NOT VALIDATED banner, as the Vocal Tract Lab Evidence tab lays it out; emitted as JSON and rendered by the Engineering Console's bundle export (the console gains an "Evidence" section fed from `vox-validation`).

### 3.4 `vox-harness` (2 days)
`voxlab run <pipeline.toml> <fixtures/>` drives any pipeline headless through a synchronous `Pipeline::run_offline` (same builder, no worker thread), writing per-tap JSONL/CSV + the provenance record. `voxlab analyze` becomes `run fingerprint.toml`. This is what the Fach Lab's Python calls.

### 3.5 Mode files (1 day)
`pipelines/fingerprint.toml` (yin → voicing → lpc → harmonics → metrics → contour; taps for the voiceprint consumer) and `pipelines/calibrate.toml` (yin → voicing; the room calibrator and the spatial RTF stage as consumers; `SpatialCal` tap for the Room screen).

### 3.6 Android and desktop wiring (1–2 days)
Delete `LegacyTail`; the shell reads every readout from taps. Desktop keeps the GPU engine (D15 is 5c) but the harness and tests run the new stages.

**Exit.** Plan gate + `LegacyTail` gone + `cargo test` on both targets + the Phase 1 gate re-run on the phone with the full pipeline (per-stage timing now includes six more stages; the 50 % budget rule still applies).

**Decisions needed.**
- Loudness: which Python reference supplies the ISO 532-1 vectors (MoSQITo, or C5's own)? Without a vectors file the stage cannot have contract tests, and the plan's rule is test-first. If no file is available at Phase 2 start, defer `loudness` to Phase 4 (Coral is its first consumer) rather than ship it untested.
- Workspace split now (recommended) or at the Coral import.
- Confirm that voiceprint/LTAS/Fach measures are consumers, not stages (a plan reading; it keeps the type table at 13).

**First prompt.** "Read Plan v3 §2–§5 Phase 2, `DECISIONS.md`, and `docs/PLAN_v3_completion.md` §3. Split the workspace first. Then, one commit per stage in the table order, wrap each kernel as a `Stage` with its existing tests as contract tests. Do not change kernel internals. Then provenance, the harness, the two mode files, and delete `LegacyTail`. Stop at the gate."

---

## 4. Phase 3 — Second backends from Vocal Tract Lab (est. 12–18 days plus waits)

**Goal (plan).** Posterior inverse as `Inverse` backend #2; MRI-reduced model + mesh as `TractModel` backend #2 producing `TractGeometry`; parity with the Kotlin app; Experiment 3 accuracy gate; Kotlin DSP retired.

**Inputs now in hand** (from the APK, `scratchpad` decompile — to be checked into `research/vocal-tract-lab-0.10.0/` with a note that it is decompiled output): `TractPosterior` (4 modes; formant→mode Jacobian, ridge 6.2, coefficient limits in SD, abstention at confidence 0.22, `maximum_relative_area_std` 0.65), `TemporalAtlasFilter` (the decay/smoothing), `TractFit` (target vs achieved resonances, RMSE, close-match flag), `ReducedModelAsset` (JSON schema 2), `TractLumenAsset` + `TractMeshCpu` (the mesh and morphing), `reduced_model.json`, `tract_lumen_v2.bin`.

### 4.1 Backends (6–8 days)
- `inverse/posterior_pca4`: `FormantTrack` → `TractParams` (4-mode vector + uncertainty). Port the Jacobian solve, limits, abstention and the temporal filter. The model is 16 kHz/1024/512 in the app but the inverse consumes formants only, so it is rate-agnostic; note this in the stage doc.
- `tract/mri_pca4`: `TractParams` → `AreaFunction` (32 sections, `area_mean + Σ q·mode`) and → `TractGeometry` (mesh from `tract_lumen_v2.bin`, morphed per the app's `TractMeshCpu`). `TractGeometry` is VTL-frame canonical per the type table; record the frame the lumen mesh is in and the transform, or record that it is unknown.
- `reduced_model.json` and `tract_lumen_v2.bin` ship as data files with provenance (`model_id`, `source_model_sha256`, `renderer_lumen_sha256` from the JSON's atlas block) — and with the atlas's own words: `scientific_release_ready: false`, held-out surface error 4.781 mm vs ≤ 4 mm target, zero independent acceptances. `vox-validation` reports these; the Evidence output carries them unchanged.

### 4.2 Parity harness (3–4 days, plus O4)
- **Without the source:** install Vocal Tract Lab on the Pixel, feed the same WAV through its replay layer (`WavPcmReplay` exists), export *its* diagnostic bundle, and compare its 500 ms `derived_state_sample` events (f0, tract confidence, abstention) against VoxLabs' bundle for the same file. Coarse, but real, and available now.
- **With the source (O4):** the app's replay fixtures and per-frame outputs through both, every tap within D5 bands. This is the gate as written. Ask for the repository again; the decompile does not replace it.

### 4.3 Experiment 3 (2–3 days)
`voxlab gen-inverse-fixtures`: ≥ 1000 held-out (F1–F3, tract-param) pairs from the forward model (`tract::resonances` over sampled `TractParams`), offline, frozen with provenance. Then median and 95th-percentile round-trip error, tract-param RMS, per-frame inverse latency on arm64 (the runner's per-stage timing gives it). Gate: 95th percentile < 5 % or < 50 Hz, < 5 ms/frame. Both bands go in `pipeline.toml` `[validation]`.

**Exit.** Plan gate; `reduced_model.json`/mesh provenance recorded; Evidence output shows the surface-error miss; Kotlin DSP marked retired in `DECISIONS.md` (D10) — or, if the inverse fails, the fallback decision (distilled offline model at the same latency bar) recorded, not a workaround.

**Decisions needed.**
- **O4:** request the Vocal Tract Lab source; meanwhile proceed from the decompile (recommended) or wait.
- **O1:** `reduced_model.json` says the atlas is a frozen metric-MRI subject mean, not VTL `.speaker` data — which may put it outside O1's GPL question. Someone has to read the corpus registry in `Vocal-Tract-Labs` and say so in writing before Phase 6 ships derived data.

---

## 5. Phase 4 — Coral import; choir branch in Rust (est. 15–20 days)

**Goal (plan).** Coral subtree; STFT, multi-F0 with harmonic cancellation, SATB labeler, JI harmony as stages; `choir.toml`; Coral's Spectrogram and Rehearsal views as tap renderers; the TypeScript worker deleted.

Coral has not been inventoried from this branch (`Bbeierle12/Coral`, private, last push 2026-09-09). The first task is the inventory; the numbers below assume a TypeScript worker with oracle fixtures, as the plan describes.

### 5.1 Inventory and subtree (2 days)
Read Coral's worker, its fixtures (0.2.0), its build. `git subtree add` into `apps/coral/` (O2, the signing key, is a release matter and does not block the import). Decide the shell layout: Tauri 2 + React at `apps/shell/`, with `vox-core` as the Rust side.

### 5.2 Stages (7–9 days)
- `stft` exists from Phase 2; port Coral's oracle fixtures as additional contract tests (windowing, bin resolution must match Coral's, or the difference is recorded as a band).
- `multi_f0/harmonic_cancellation`: `Spectrum` → `NoteSet`.
- `satb/labeler`: `NoteSet` → `SectionLabels`.
- JI harmony: `SectionLabels` + `NoteSet` → a consumer (it is a display computation, not a wire), unless Coral's fixtures show it needs to be a stage.
- `pipelines/choir.toml`.

### 5.3 Shell (5–7 days)
The Tauri/React shell is stood up here: taps over a Tauri channel to React; Coral's Spectrogram and Rehearsal views rendered from `Spectrum` and `NoteSet` taps. Desktop only in this phase (the shell reaches the phone in 5a). The egui desktop build stays as-is until 5c.

**Exit.** Coral 0.2.0 fixtures reproduce within tolerance from the Rust branch; the TypeScript worker is deleted in the same PR; `choir.toml` runs headless in `vox-harness`.

**Decisions needed.** Shell repo layout (subtree path names); whether Coral's *audio capture* (browser `getUserMedia`, if that is what it does) is replaced by cpal in this phase or at 5a (D6 says all capture is native; doing it here avoids two capture paths for one phase).

---

## 6. Phase 5a — Android under the Tauri shell (est. 12–15 days)

**Goal (plan).** Live Model in the Tauri Android build on the Pixel with the Phase 1 latency numbers; Experiment 2 (WebGL2 vs WebGPU in the Pixel WebView); native-vs-desktop tolerance bands committed.

### 6.1 Toolchain (2 days)
Tauri 2 Android needs JDK, Android Studio SDK/NDK (the NDK r26d already used), `cargo tauri android init`. The `build-dev-apk.sh` convention (`org.voxlabs.core.dev`, "VoxLabs (dev)", 16 KB alignment, signature check) is re-implemented over `cargo tauri android build`; keep the `.dev` id so both apps coexist.

### 6.2 Core under Tauri (4–5 days)
- `vox-core` as the Tauri app's Rust library; no `android_main` — Tauri's activity owns the process. The runner, ring buffer and taps are unchanged (D9: shell-agnostic).
- cpal: manual `ndk_context::initialize_android_context` (O6). Permission: a Tauri plugin (or the existing JNI `permission.rs` against Tauri's activity — the `with_activity` helper takes the activity from `ndk_context`, so it may work unchanged; verify).
- Diagnostics: the runtime and store are shell-agnostic; the Engineering Console page is re-implemented in React (same sections, same copy) and fed by Tauri commands over `diagnostics::runtime`. The egui console retires with the egui Android build.
- Everything Android-specific in JNI (`device_probe`, `spatial`, `share_intent`, `permission`, `diagnostics::export`) is checked against Tauri's activity context; expect small fixes, not rewrites.

### 6.3 Experiment 2 (2 days)
`navigator.gpu` probe + three.js scene with the production airway mesh (from Phase 3's `TractGeometry`) and morph targets, WebGL2 vs `WebGPURenderer`, in the Pixel's System WebView, 43 ms cadence, 2 minutes; sustained fps and dropped frames. Decision rule is in the plan.

### 6.4 Tolerance bands (2 days)
Same fixture through arm64 native and x86_64 desktop via `vox-harness`; every tap's delta recorded; bands set from data into `pipeline.toml` (`[tolerance]`). D5.

**Exit.** Plan gate; a `docs/phase5a-gate.md` with the numbers next to Phase 1's; the egui Android build retired (the cargo-apk path can remain in the repo until 5c for Resonator's sake, O3).

**Decisions needed.** Confirm phone-under-Tauri precedes desktop-under-Tauri (the plan's order; the alternative, 5c first, would give the desktop D15 engine sooner at the cost of D18).

---

## 7. Phase 5c — Desktop product under the Tauri shell; D15 (est. 10–14 days)

**Goal (plan).** The desktop engine is the pipeline on the CPU stages; GPU YIN removed; spectrogram/scope/calibration re-homed; Live Model, Fingerprint, Choir from the same TOMLs as the phone; egui desktop retired (or kept for Resonator only).

### 7.1 Freeze the GPU baseline (1 day — do it at Phase 2 start)
`voxlab freeze-gpu-baseline`: run the current `AnalysisEngine` over the fixture set on a machine with a wgpu adapter and write its per-frame outputs as the frozen fixture. This can only run where wgpu renders (not this container); it must be done **before** Phase 2 rewires anything on desktop, because the plan's D15 gate compares against it.

### 7.2 D15 execution (5–7 days)
- Replace `analysis::AnalysisEngine` with the runner on the desktop entry point; the analysis thread becomes the runner's worker. (D15 verification note: the desktop live UI depends on the GPU path today, and `gpu_yin_matches_cpu` is the only test that does — it becomes the baseline comparison.)
- Delete `yin_diff.wgsl`, `yin_scan.wgsl`, `math::yin_f0_from_diff_cumsum`, `pollster`; drop `wgpu` from the analysis dependencies (eframe/Tauri rendering decides whether the crate stays).
- Spectrogram is already a stage from Phase 2; scope is a tap consumer; room calibration runs `calibrate.toml`.

### 7.3 Desktop product pass (4–6 days)
File import at scale and multi-file batch through `vox-harness` from the shell; larger-screen layouts for the same tap views; the Engineering Console in React on desktop (shared with 5a).

### 7.4 Tolerance (1 day)
x86_64 vs arm64 through the harness; bands committed (extends 5a's table).

**Exit.** Plan gate: frozen-baseline match within `gpu_yin_matches_cpu` tolerance, `voxlab` harness bit-for-bit, three modes from the same TOMLs, egui retired or scoped to Resonator (O3 answered).

---

## 8. Phase 5b — Browser wasm (optional; est. 5–8 days for the experiment)

Experiment 1 as written: the C2 kernel per 1024-hop in a Web Worker for 60 s; `rustfft` `wasm_simd` and fallback profiles; zero misses over 10 min at < 50 % budget. Output is an experiment log and a decision on O5, never a workaround. The wasm build already type-checks (`cargo check --target wasm32-unknown-unknown`), so the experiment is mostly threading (`+atomics`, COOP/COEP) and a worker host in `vox-wasm`. Schedule it whenever a week is free; nothing waits on it.

---

## 9. Phase 6 — Workbench + VTL backend (desktop only; est. 15–20 days)

`vox-tract-vtl` behind a feature flag: VocalTractLab (GPL) formant fitting as `Inverse` backend #3, VTL geometry as `TractModel` backend #3, 19-D `TractParams`. C5's Python generates inverse tables and PCA bases through `vox-harness`; outputs frozen as data files with provenance. Workbench mode: any pipeline with every tap exposed and two backends side-by-side in one slot.

Gate as written, plus CI: `cargo tree` and a symbol check proving the production build has no VTL symbols and no GPL code. **O1 must have a written answer before this phase ships anything derived.** The Vocal Tract Lab data (Phase 3) is separate from this question unless the corpus registry says it descends from `.speaker` files.

---

## 10. Phase 7 — Anatomy (open-ended; plan v2 Phase 6 criteria)

`vox-anatomy`: registration into the VTL frame, full-head resonance chamber geometry, morph targets for the three.js renderer. Inputs: `TractGeometry` from Phase 3, the `Vocal-Tract-Labs` corpus (registry + adapters; datasets stay external per its data policy). Surface-error target in config; reported by `vox-validation` in the Evidence output. Not estimated here: the scope depends on which corpus sources are admitted, which is a research decision.

---

## 11. Phase 8 — External workbench

Gated on external use exactly as the plan says. Nothing to schedule.

---

## 12. Cross-cutting

- **CI (Phase 2, day 1).** The repository has no workflows. Add one: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo check --target aarch64-linux-android --lib`, `cargo check --target wasm32-unknown-unknown --lib`, the `pipeline.toml` drift test, and (Phase 6) the GPL symbol check. The APK build stays local until a keystore lives in CI secrets.
- **Fach Lab M1–M5** runs on `vox-harness` in parallel from Phase 2 on; M4 ("into the app") lands on the Tauri shell after 5c.
- **Numbers rule.** Every new threshold (tolerance bands, Experiment 3 bands, loudness constants, validation gates) goes in `pipeline.toml` under its own table, drift-tested, as `[diagnostics]` was.
- **Sub-500-line files.** `math.rs` (2709 lines), `spatial.rs` (1360), `device_probe.rs` (1054), `fach.rs` (849), `analysis.rs` (573), `android.rs` (551) are over. Phase 2's wrapping is the natural moment to split `math.rs` by stage (the wrappers already point at the seams); `analysis.rs` goes in 5c; the others when touched.
- **Docs per phase.** A `docs/phaseN-gate.md` with the procedure and, after the run, the numbers; `DECISIONS.md` status notes for every D/O touched; `STATUS.md` addendum at each gate.

---

## 13. Decisions and inputs needed from Brandon, in the order they block

| # | Needed by | Question | Recommendation |
|---|---|---|---|
| 1 | 1-close-out | adb on a desktop for the ten-minute gate and on-device contract tests? | Yes; it is also required for Tauri Android in 5a |
| 2 | Phase 2 start | Workspace split now or at Coral import | Now |
| 3 | Phase 2 | Source of ISO 532-1 reference vectors | Name the file; else defer `loudness` to Phase 4 |
| 4 | Phase 2 | Voiceprint/LTAS/Fach as tap consumers, not stages | Consumers |
| 5 | Phase 2 start | Freeze the GPU baseline on a wgpu machine (for 5c's gate) before desktop changes | Do it first; it cannot be done later |
| 6 | Phase 3 | O4: Vocal Tract Lab source repository | Request it; start from the decompile meanwhile |
| 7 | Phase 3 / 6 | O1: provenance of `reduced_model.json` and any `.speaker`-derived data | Read the corpus registry; write the answer in `DECISIONS.md` |
| 8 | Phase 4 | Shell layout (`apps/shell`, `apps/coral`) and whether Coral's capture moves to cpal in 4 or 5a | Move it in 4 |
| 9 | Phase 5a | Confirm 5a before 5c (phone first) | As the plan says |
| 10 | Phase 5c | O3: does Resonator keep the egui build alive | Decide at 5c; nothing depends on it earlier |

---

## 14. Rough calendar

Sequential, one developer, working days, from the day the microphone fault is closed:

| Phase | Days | Cumulative |
|---|---|---|
| 1 close-out | 2–5 | 5 |
| 2 | 15–20 | 25 |
| 3 | 12–18 (+ O4 wait) | 43 |
| 4 | 15–20 | 63 |
| 5a | 12–15 | 78 |
| 5c | 10–14 | 92 |
| 5b (optional) | 5–8 | — |
| 6 | 15–20 | 112 |
| 7 | open | — |

About four and a half months to the end of 5c; six to the end of Phase 6. The Fach Lab milestones overlap rather than add. The estimates carry the usual risk that the first on-device run of any phase finds something the host did not (Phase 1's microphone fault is the example).
