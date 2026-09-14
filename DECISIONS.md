# Decisions

Copied verbatim from `docs/PLAN_V3_pipeline.md` §1 (Plan v3, 2026-09-07). Every
row is the table as written there; status changes are recorded here first and
mirrored back to the plan.

| # | Decision | Status |
|---|---|---|
| D1 | One monorepo rooted in VoxLabs; Coral imported by subtree; Tauri 2 + React shell over Rust core (shell option A) | Decided (v2) |
| D2 | Crates: `vox-core`, `vox-tract`, `vox-tract-vtl` (desktop, feature-gated), `vox-anatomy`, `vox-validation`, `vox-harness`, `vox-wasm` | Decided (v2) |
| D3 | VTL only in desktop Workbench; production ships derived data | Decided (v2); data-license question still Open (O1) |
| D4 | C7 choir analysis moves from TypeScript to Rust sharing the C2 core | Decided (Sept 8 review) |
| D5 | Cross-target numeric tests use tolerance bands, never bit-equality. Default band for F1–F3 lives in `pipeline.toml` (report suggests tenths of a Hz for kernel parity; the inverse gate uses a looser perceptual band, see Phase 3) | Decided (Sept 8 review, float non-determinism) |
| D5a | No `fast-math` on any target. Explicit multi-lane accumulators (the 8-lane pattern from the benchmark) are the vectorization mechanism, and the lane count is config | Decided (Sept 8 review, Criterion 8) |
| D5b | Linear algebra via `faer` or `nalgebra` core only. `ndarray-linalg` is banned (x86-only, wasm build failure #121) | Decided (Sept 8 review, C3) |
| D6 | Audio I/O (C1) is *not* a pipeline stage. Callback writes a ring buffer only. DAG runs on a worker per hop. Ring buffer must re-frame to hop 1024 itself because Android may ignore `BufferSize::Fixed` (cpal #902). All capture goes through cpal native; nothing captures through the WebView (`getUserMedia` permission cluster, Tauri #10846/#10898) | Proposed |
| D7 | `Stage` trait + DAG runner live in `vox-core`; `TractModel` conforms to `Stage` | Proposed |
| D8 | Every stage is allocation-free after `init()`; runner preallocates all intermediate buffers at pipeline build | Proposed |
| D9 | Pipeline definitions are serialized TOML; one file per app mode; shells load a definition and render taps | Proposed |
| D10 | Vocal Tract Lab's posterior inverse and MRI-reduced model are the target designs for `Inverse` backend #2 and `TractModel` backend #2; Kotlin DSP retires after tolerance-band parity | Decided (v2) |
| D11 | Provenance record per run (stage impls, versions, params, target, build flags, input hash) is the input to `vox-validation`; output format is Vocal Tract Lab's Evidence tab | Proposed |
| D12 | No pipeline editor before Phase 8, and Phase 8 is gated on external use | Proposed |
| D13 | Rename `voice_harmonic_engine` → `vox-core` and split `ui.rs` *before* the Coral subtree import | Proposed |
| D14 | Hop 1024 is canonical for the whole pipeline; spectrogram moves to hop 1024 with `overlap` as its own parameter | Decided |
| D16 | LPC order is adaptive (8–20 after decimation to 11 025 Hz) by design; not fixed 24 | Decided |
| D17 | `parabolic_flat_eps` (1e-6 YIN vs 1e-12 elsewhere) and synthesis vs analysis default bandwidths: confirm intentional or reconcile, from the code | Decided (check in Phase 1) |
| O1 | Whether inverse tables and PCA bases derived from VTL `.speaker` files are GPL-encumbered | Open — check the data files' license in the VTL repo; get a real opinion before any sale |
| O2 | Canonical Coral signing key | Open |
| O3 | Whether Resonator adopts the core (cheap if it's a 3-stage pipeline config) | Open |
| O4 | Vocal Tract Lab source repo not yet provided | Open — blocks Phase 3 |
| O5 | wasm threading path for Rust (nightly + `build-std` + `+atomics`, COOP/COEP for SharedArrayBuffer) — needed for the analysis worker, not just the audio thread. Report's fallback rule: if Rust fails Experiment 1 and Emscripten C++ passes, adopt Emscripten for the web audio thread *only*; the core stays Rust | Open — Phase 5 experiment |
| O6 | cpal on Android under Tauri's harness needs manual `ndk_context::initialize_android_context` (cpal #720) — integration cost, not a blocker | Open — Phase 5 |

## Status notes (kept below the table so the table stays as the plan wrote it)

- **O1** (2026-09-08): unknown — needs the data-file license check in the VTL
  repository before any answer is recorded; no opinion has been obtained.
- **D13** (2026-09-08): carried out in Phase 0, steps (a) and (b) — crate
  renamed to `vox-core`, Android package to `org.voxlabs.core`, `ui.rs` split
  into `src/ui/`. Its row above still reads "Proposed" because that is how §1
  is written; the plan is the place to change it.
- **D14, D16, D17** (2026-09-08): copied from Plan v3 §1 as written (D15 and
  D18 exist in the plan but were not requested here). D14 is applied to the
  runner (`pipelines/live_model.toml`, hop 1024); the spectrogram still runs at
  hop 512 with its own `[spectrogram]` keys until it becomes a stage (Phase 5c).
  D16 is the code as it stands: `lpc.order_base + fs_dec / lpc.order_hz_per_pole`
  clamped to `[lpc.order_min, lpc.order_max]` = 8–20 at 11 025 Hz.
- **D17 findings** (2026-09-08, read from the code and history; nothing
  changed):
  - `yin.parabolic_flat_eps = 1e-6` vs `1e-12` in `hnr`, `perturbation`, and
    `vibrato`. All four guard the same formula, `|s0 − 2·s1 + s2|`. YIN's runs
    on the cumulative-mean-normalized difference, whose values are of order 1,
    so 1e-6 is a real "flat neighbourhood" threshold at f32 precision. The
    other three run on raw signal-scale quantities (autocorrelation sums,
    sample peaks, DFT power) where 1e-12 only catches an exact-zero
    denominator. Both were introduced together in commit b32aed4
    (2026-07-01, "publish jitter/shimmer/CPP on both analysis paths") with no
    comment either way. Verdict: consistent with intent (different scales), but
    undocumented; reconciling to one value would change YIN's lag refinement.
    Recommendation: keep both, document the scale argument on the fields.
  - Synthesis default bandwidths `[50, 100, 150]` vs analysis `[80, 120, 160]`.
    The synthesis triple is the oscillator's start-up envelope before any
    profile arrives (narrower = more resonant idle tone); the analysis triple is
    the envelope held until the first voiced frame. They can both reach the
    oscillator: `set_profile` adopts the analysis defaults whenever a voiced
    frame's LPC resolves no F1. Both also date from b32aed4 with no comment.
    Verdict: cannot be confirmed intentional from the code; the values do not
    interact numerically (the glide converges either way), so no behavior
    hinges on it. Recommendation: reconcile to the analysis triple when Phase 2
    wraps synthesis, since that is the envelope the analyzer actually holds.

- **D18 amendment — Pixel only** (2026-09-14, Brandon: "This is going on my
  Google Pixel right now. Nothing else."): the phone is the only product.
  Desktop stays a build-and-test target for the contract tests and the
  `voxlab` harness, not a product; Phase 5c reduces to the D15 engine
  cleanup on desktop; Phase 5b (browser wasm) is dropped from the schedule
  (the wasm build must still type-check, nothing more). Every gate is the
  phone's.
- **Completion-plan calls** (2026-09-14, made by the agent under the
  "complete all phases" directive, per `docs/PLAN_v3_completion.md` §13;
  each stands until Brandon says otherwise):
  1. No adb on the study phone: the contract tests run on device from the
     Engineering Console self-test (`pipeline::contract`), the runner's
     timing report is a `runtime/pipeline_report` event in the bundle.
  2. Workspace split at the start of Phase 2.
  3. No ISO 532-1 reference vectors are available; the `loudness` stage is
     deferred until a vectors file exists (test-first rule).
  4. Voiceprint, LTAS and the Fach measures are tap consumers, not stages.
  5. The GPU baseline cannot be frozen here (no wgpu adapter) and desktop is
     not a product; D15's gate becomes: the desktop CPU engine equals the
     `voxlab` harness bit-for-bit and `gpu_yin_matches_cpu`'s CPU side.
  6. O4: Phase 3 proceeds from the decompiled Vocal Tract Lab 0.10.0 APK;
     the source is still wanted for fine-grained parity.
  7. O1: to be answered from the `reduced_model.json` atlas block and the
     `Vocal-Tract-Labs` corpus registry in Phase 3.
  8. Coral capture moves to cpal in Phase 4.
  9. Phone first (5a before any desktop work), unchanged.
  10. O3 (Resonator/egui) decided at 5c.
- **Phase 2 status** (2026-09-14): D6, D7, D8, D9 and D11 are implemented
  as proposed — audio I/O stays outside the pipeline (ring buffer +
  runner), `Stage` and the runner live in `vox-core`, every stage
  preallocates at `init`, one TOML per mode (`live_model`, `fingerprint`,
  `calibrate`), and a provenance record per run feeds `vox-validation`,
  whose Evidence output follows the Vocal Tract Lab Evidence tab. The rows
  above keep the plan's wording; see `docs/phase2-gate.md`.
- **Phase 3 status** (2026-09-14): D10's two backends exist —
  `inverse/posterior_pca4`, `tract/mri_pca4` and the `mesh/lumen_v2`
  stage producing `TractGeometry`, ported formula-for-formula from the
  decompiled Vocal Tract Lab 0.10.0 APK with the app's formulas as their
  contract tests (`docs/phase3-gate.md`). The data files ship in
  `assets/vocal_tract_lab/` with digests checked at load and their own
  release statements carried into provenance and the Evidence output.
  D10's second clause — "Kotlin DSP retires after tolerance-band parity" —
  is **not met**: parity is bundle-against-bundle on the Pixel and has not
  been run. The Kotlin DSP is not marked retired.
- **Experiment 3 verdict and fallback** (2026-09-14): `grid_story` meets
  the bands (p95 1.9 % / 12 Hz, 3 µs); `posterior_pca4` meets latency
  (0.2 µs) and misses accuracy (p95 42 % over ±2 SD, 9.6 % within
  ±0.5 SD, after F4 was fed in) under VoxLabs' forward model, whose
  Jacobian is within 19 % of the app's own. The map is the exact inverse
  of the app's 4 × 4 Jacobian and is a local linearization by the
  app's own description. Decision, per the plan's exit clause: the
  posterior is kept as the **parity target** (it is the app's
  algorithm) and is not offered as an accuracy-grade inverse;
  `live_model` keeps the Story grid; the plan's fallback (a distilled
  offline-trained model at the same latency bar, report C3) is the path
  if an accuracy-grade atlas inverse is wanted, and is not started until
  the phone parity comparison says whether the port matches the app.
- **F4** (2026-09-14): `FormantTrack` gained `f4: Option<Formant>` (the
  fourth in-band LPC pole, from the same candidate list F1–F3 come from)
  because the app's map needs four formants (`docs/phase3-gate.md`, "Why
  F4 matters"). `VocalProfile`, the synthesizer and every three-formant
  path are unchanged.
- **O4** (2026-09-14): resolved as far as the decompile allows. Ported:
  `ReducedModelAsset`, `TemporalAtlasFilter`, the evidence score from
  `FrameAnalyzer`, `TractLumenAsset`, `TractMeshCpu`,
  `ArticulatorPosterior`. Still wanted from the source: the Kotlin
  originals (comments, tests), the replay fixtures and per-frame outputs
  for the fine-grained parity gate, `SharedTractModel.fit`/`TubeGrid`
  semantics. The decompile is checked in under
  `research/vocal-tract-lab-0.10.0/` and labelled as such.
- **O1** (2026-09-14, from the files in hand — the `Vocal-Tract-Labs`
  corpus registry was not available to read): `reduced_model.json`'s
  atlas block says the model is the app's own frozen (0.6.3) metric-MRI
  engineering mean over five subject means; the app's MRI reference
  (`reference.js`) cites Ruthven, Peplinski and Miquel (2023), Zenodo
  10046815, CC BY 4.0, 2-D real-time MRI labels. No VocalTractLab
  (Birkholz, GPL) `.speaker` data is involved in Phase 3's files, so
  O1's GPL question does not attach to them. O1 stays open for Phase 6
  (the `vox-tract-vtl` backend), where it does attach.

