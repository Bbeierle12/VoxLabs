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
