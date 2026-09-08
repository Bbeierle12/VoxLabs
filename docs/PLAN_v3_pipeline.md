# VoxLabs Plan v3 — the pipeline amendment to Plan v2

*2026-09-07. This amends Plan v2; it does not replace it. Crate layout, shell choice, GPL boundary, and merge decision carry over. What v3 adds is the seam v2 left undefined: how the stages inside `vox-core` relate to each other. Everything tagged **Decided** is from v2 or the Sept 8 architecture review. **Proposed** needs Brandon's sign-off. **Open** is unresolved.*

---

## 0. What v3 changes, in one paragraph

Plan v2 already made `TractModel` a trait with two backends. v3 applies that pattern to every stage in the analysis chain — pitch, formants, harmonics, inverse, smoothing, tract, loudness, multi-F0 — and connects them with a preallocated DAG runner whose every wire can be tapped. The app modes (Fingerprint, Live Model, Resonance Lab, Choir, Calibrate, Workbench) become saved pipeline definitions over one core. No new crate. No editor. The visual layer stays Coral's spectrogram view plus per-wire plots.

Why this and not the general glyph platform: the prior-art research killed the general version; the narrow version survives only where the type universe is small and the data is perceptible. Vocal analysis is that case. The doc's sections 6, 15, 35, 36, 37 are the ones being built; the rest are not.

---

## 1. Decisions

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
| D13 | Rename `voice_harmonic_engine` → `vox-core` and split `ui.rs` *before* the Coral subtree import | Done (Phase 0, PR #1) |
| D14 | Hop 1024 is canonical for the whole pipeline; spectrogram moves to hop 1024 with `overlap` as its own parameter | Decided |
| D15 | `analysis::AnalysisEngine` + GPU YIN path removed; desktop engine becomes the pipeline on `FrameAnalyzer`, reversing the earlier "report Unavailable, don't fall back" ruling. Freeze GPU fixture output first; gate new engine against it within `gpu_yin_matches_cpu` tolerance and against the `voxlab` harness bit-for-bit. Remove `pollster`; remove `wgpu` only if eframe doesn't render with it | Decided — lands in Phase 5c, not Phase 1 |
| D16 | LPC order is adaptive (8–20 after decimation to 11 025 Hz) by design; not fixed 24 | Decided |
| D17 | `parabolic_flat_eps` (1e-6 YIN vs 1e-12 elsewhere) and synthesis vs analysis default bandwidths: confirm intentional or reconcile, from the code | Decided (check in Phase 1) |
| D18 | Phone and desktop are both products. **Phone first**: the Pixel 10 Pro XL running the Rust core native arm64 is the target for Phases 1–5a, and every latency gate in those phases is measured there. Desktop must build and pass contract tests throughout, and gets its own product pass in Phase 5c. Browser wasm is secondary (Phase 5b) | Decided |
| O1 | Whether inverse tables and PCA bases derived from VTL `.speaker` files are GPL-encumbered | Open — check the data files' license in the VTL repo; get a real opinion before any sale |
| O2 | Canonical Coral signing key | Open |
| O3 | Whether Resonator adopts the core (cheap if it's a 3-stage pipeline config) | Open |
| O4 | Vocal Tract Lab source repo not yet provided | Open — blocks Phase 3 |
| O5 | wasm threading path for Rust (nightly + `build-std` + `+atomics`, COOP/COEP for SharedArrayBuffer) — needed for the analysis worker, not just the audio thread. Report's fallback rule: if Rust fails Experiment 1 and Emscripten C++ passes, adopt Emscripten for the web audio thread *only*; the core stays Rust | Open — Phase 5 experiment |
| O6 | cpal on Android under Tauri's harness needs manual `ndk_context::initialize_android_context` (cpal #720) — integration cost, not a blocker | Open — Phase 5 |

---

## 2. The type universe

This is the whole reason the idea works here. Keep it small. Adding a type requires a line in this table.

| Type | Contents | Produced by | Consumed by |
|---|---|---|---|
| `AudioFrame` | f32 samples, sample rate, frame index | ring-buffer reader | STFT, YIN, loudness |
| `Spectrum` | complex bins, bin resolution | STFT | harmonics, formants(cepstral), spectrogram tap |
| `F0Track` | Hz, confidence, voiced flag | YIN / pYIN / CREPE | harmonics, smoother, multi-F0 seed |
| `HarmonicSeries` | H1–H16 amplitude, phase | Goertzel | voiceprint, HNR, H1–H2, tilt |
| `VoiceMetrics` | HNR, CPP, jitter, shimmer, centroid, tilt, LTAS | metrics stages | voiceprint, Fach measures |
| `FormantTrack` | F1–F4 Hz + bandwidths, confidence | LPC/Levinson / cepstral / other | inverse, vowel estimate |
| `TractParams` | model-specific vector (2-mode Story; 4-mode PCA; 19-D VTL) + uncertainty | inverse | smoother, TractModel |
| `AreaFunction` | N sections, lengths, areas | TractModel | resonance solver, 2.5D tube |
| `TractGeometry` | centerline, cross-sections, surface mesh — **VTL frame canonical** | TractModel / anatomy | renderer |
| `Loudness` | ISO 532-1 sones, specific loudness | Zwicker stage | Coral, calibrate |
| `NoteSet` | multi-F0 notes with confidence | multi-F0 | SATB labeler |
| `SectionLabels` | S/A/T/B assignment | labeler | JI harmony |
| `SpatialCal` | two-mic delays / gains | calibrate stage | Room screen |

Rule: a stage declares exactly which of these it consumes and produces. The runner refuses to connect mismatched wires and names the missing adapter chain.

---

## 3. The Stage contract (sketch, not final)

```rust
pub trait Stage: Send {
    type In;
    type Out;
    /// Called once. All allocation happens here. Config is immutable after.
    fn init(cfg: &StageConfig, fmt: &StreamFormat) -> Result<Self, StageError> where Self: Sized;
    /// Called per hop on the worker. No allocation. No I/O. No panics on bad input — return Err.
    fn process(&mut self, input: &Self::In, out: &mut Self::Out) -> Result<(), StageError>;
    /// Identity for provenance: name, semver, backend id.
    fn ident(&self) -> StageIdent;
}
```

Every existing validated function in VoxLabs becomes a `Stage` impl by *wrapping*, not rewriting. Its existing tests become that stage's contract tests. Internals get reorganized only after the wrapper is green.

Constraints inherited from Brandon's rules: config-driven (every number in `pipeline.toml` or the stage's `StageConfig`), fail-loud (`Err`, never a silent default), sub-500-line files, test-first per phase.

---

## 4. Config

Complexity budget: solo developer, one product → `const` defaults in code + one `pipeline.toml` per mode. No env vars, no runtime override system.

`pipeline.toml` holds, per mode: stream format (48 kHz, frame 2048, hop 1024), the ordered stage list with backend ids, per-stage params (LPC order 24, YIN threshold, section count 44, inverse cadence ~43 ms, smoothing constants, decay), tap list, and target-specific tolerance bands. The current magic numbers in `tract.rs`, `ui.rs`, and Coral's worker are the extraction list for Phase 1.

---

## 5. Phases and gates

Each phase: write acceptance tests first; gate must pass before the next phase starts; anything that would break a gate is a stop, not a workaround.

### Phase 0 — Housekeeping (before any pipeline code)
- Rename crate/package (D13). Split `ui.rs` into per-screen files under 500 lines.
- `DECISIONS.md` with the table in §1. Record O1 status honestly ("unknown, needs data-file license check").
- Extract magic numbers from `tract.rs` and the DSP modules into `StageConfig` structs + `pipeline.toml`. No behavior change.
- **Gate:** existing test suite passes unchanged; `cargo build` on desktop and Android.

### Phase 1 — Walking skeleton: Live Model as a pipeline
- `Stage` trait, `Pipeline` builder (validates wires against §2, preallocates), `Runner` (worker thread, per-hop), `Tap` (bounded channel to the shell).
- Wrap: YIN → `F0Track`, LPC/Levinson → `FormantTrack`, existing 41×41 grid → `TractParams`, Story `TractModel` → `AreaFunction` + 2.5D tube.
- Pipeline `live_model.toml`: ring buffer → YIN, LPC → grid inverse → Story tract → tube view.
- Shell for this phase: the existing cargo-apk/egui Android build and the egui desktop build. The Tauri/React shell arrives with the Coral import in Phase 4. The pipeline is shell-agnostic (D9); do not stand up Tauri here.
- Desktop: keep the existing GPU engine untouched this phase. Desktop must build and pass all contract tests; the desktop live engine swap (D15) happens in Phase 5c.
- **Gate (measured on the Pixel 10 Pro XL, `org.voxlabs.core.dev` build):** sings into mic, tube moves; each tap shows a live value in a debug panel; end-to-end mic-to-render latency logged against the threshold in config, with per-stage timing from the runner; zero hop deadline misses over 10 min (worst-case hop < 50 % of the 21.3 ms budget — the report's Experiment 1 threshold, applied natively); all wrapped stages pass their prior tests as contract tests; wiring a wrong type fails at build with the adapter suggestion. Desktop must build and pass the same contract tests but is not the latency gate.

### Phase 2 — Wrap the rest of C2; provenance; validation
- Wrap Goertzel, HNR, CPP, jitter/shimmer, tilt, LTAS, voiceprint, Fach measures. Add ISO 532-1 loudness stage with the Python reference's vectors as contract tests (answers "where psychoacoustic lives").
- Provenance record (D11) written per run. `vox-validation` reads it and emits Evidence-format output.
- `vox-harness` runs any pipeline headless over a fixture directory (this is what C5's Python pipeline will call).
- Pipelines: `fingerprint.toml`, `calibrate.toml`.
- **Gate:** Fingerprint mode reproduces enrollment/match results on the `voxlab` synthetic fixtures within tolerance; provenance round-trips (record → rebuild pipeline → same result within band).

### Phase 3 — Second backends from Vocal Tract Lab (blocked on O4)
- Port the posterior inverse (decay, uncertainty) as `Inverse` backend #2; port the 4-mode MRI-reduced model + surface mesh as `TractModel` backend #2 producing `TractGeometry`.
- Parity harness: same fixtures through Kotlin app (via its replay layer) and Rust pipeline; compare every tap with tolerance bands (D5).
- Inverse accuracy test (report Experiment 3): ≥1000 held-out (F1–F3, tract-param) pairs generated offline; measure median and 95th-percentile formant round-trip error, tract-param RMS error, and per-frame inverse latency on arm64.
- **Gate:** parity within bands on the fixture set; 95th-percentile F1–F3 round-trip error within the perceptual band in config (report default: <5 % or <50 Hz) at <5 ms/frame on arm64; held-out surface error reported by `vox-validation` (currently 4.781 mm vs ≤4 mm — report it, don't hide it); Kotlin DSP marked retired. If the inverse fails the accuracy gate, the fallback is a distilled offline-trained model evaluated against the *same* latency bar — not a live differentiable model (report C3).

### Phase 4 — Coral import; choir branch in Rust (D4)
- Subtree Coral. Port STFT (with its oracle fixtures as contracts), multi-F0 with harmonic cancellation, SATB labeler, JI harmony as stages. `choir.toml`.
- Coral's Spectrogram and Rehearsal views become tap renderers over the shared shell.
- **Gate:** Coral 0.2.0 fixtures reproduce within tolerance from the Rust branch; the TypeScript worker is deleted, not kept "just in case."

### Phase 5a — Android under the Tauri shell (primary target)
- The Rust core runs **native arm64** under Tauri 2 Android; the WebView is UI only. No wasm on the phone.
- Wire cpal under Tauri's Android harness: manual `ndk_context::initialize_android_context` (O6, cpal #720). All capture stays native; nothing through WebView `getUserMedia` (D6).
- Experiment 2 (report): `navigator.gpu` probe + three.js scene with the production airway mesh and morph targets, WebGL2 vs `WebGPURenderer`, in the Pixel's System WebView, animated at the 43 ms cadence for 2 min. Metric: sustained fps, dropped frames.
- Native-vs-desktop tolerance: same fixture through arm64 native and desktop x86_64; every tap compared within the D5 bands. Record observed deltas; set the bands from data.
- **Gate:** Live Model runs in the Tauri Android build on the Pixel with the same latency numbers as the Phase 1 egui build (no regression from the shell swap). Experiment 2 — WebGL2 ≥ 55 fps sustained → keep WebGL2; < 30 fps with `navigator.gpu` present → pursue WebGPU; < 30 fps and no WebGPU → reduce mesh/morph complexity. Tolerance bands committed to `pipeline.toml`.

### Phase 5c — Desktop product under the Tauri shell
- Execute D15: freeze the GPU engine's fixture output, replace `analysis::AnalysisEngine` with the pipeline on `FrameAnalyzer`, re-home spectrogram (stage), scope (tap consumer), room calibration (wired to `calibrate.toml` from Phase 2). Delete `yin_diff.wgsl`, `yin_scan.wgsl`, `math::yin_f0_from_diff_cumsum`, `pollster`; `wgpu` only if eframe doesn't render with it.
- Desktop-specific product features that the phone doesn't carry: file import at scale, multi-file batch through `vox-harness`, larger-screen layouts for the same tap views.
- Desktop-vs-phone tolerance: same fixture through x86_64 and arm64; every tap within D5 bands; bands committed.
- **Gate:** new desktop engine matches the frozen GPU baseline within `gpu_yin_matches_cpu` tolerance and the `voxlab` harness bit-for-bit; Live Model, Fingerprint, and Choir modes run on desktop from the same pipeline TOMLs as the phone with no mode-specific code paths; egui desktop build retired (or kept only for Resonator per O3).

### Phase 5b — Browser wasm target (secondary; may be deferred indefinitely)
- `vox-wasm` hosts the `Runner` in a Web Worker. Build constraint: `rustfft`/`realfft` `wasm_simd` at compile time; two build profiles, SIMD and fallback.
- Experiment 1 (report): C2 kernel per 1024-hop in a browser, 60 s continuous. Rust→wasm worker first; Emscripten C in the audio worklet only if Rust fails (O5 fallback rule).
- Three-way tolerance: add wasm32 to the Phase 5a comparison. This is where FMA/denormal divergence shows up.
- **Gate:** zero deadline misses over 10 min, worst-case frame < 50 % of budget. Any failure produces an experiment log and a decision on O5, not a workaround. This phase does not block Phases 6–7.

### Phase 6 — Workbench + VTL backend (desktop only)
- `vox-tract-vtl` behind a feature flag: VTL formant-fitting as `Inverse` backend #3, VTL geometry as `TractModel` backend #3, 19-D `TractParams`.
- Offline: C5 Python generates inverse tables and PCA bases using `vox-harness`; outputs frozen as data files with provenance.
- Workbench mode = any pipeline with every tap exposed, side-by-side backend comparison for one slot (two `Inverse` impls, overlaid tracks).
- **Gate:** production build contains no VTL symbols and no GPL code (`cargo tree` + symbol check in CI); shipped derived data carries its provenance; O1 has a written answer.

### Phase 7 — Anatomy
- `vox-anatomy` stages: registration into VTL frame, full-head resonance chamber geometry, morph targets for the three.js renderer.
- **Gate:** per v2 Phase 6 criteria; surface-error target in config, reported by validation.

### Phase 8 — External workbench (only if gated in)
- Gate to *start*: at least two people who are not Brandon have asked to run their own recordings through it, or Brandon has reached for the Workbench over Praat/Python for a month.
- Then: batch mode over a directory, Praat-parity on speech for LPC formants, export of provenance with figures. Still no node editor; a pipeline picker and per-stage parameter panel is the UI.
- If the gate never opens, Phase 8 does not exist and nothing is lost.

---

## 6. Things this plan deliberately does not do

- No graph editor, no glyph registry, no marketplace, no multi-language codegen, no legacy-code import, no "software genome." Dead per the research.
- No neural pitch/formant backend before Phase 6 — deterministic first, per the Vocal Tract Core draft principle v2 adopted.
- No egui retention beyond Resonator unless O3 says otherwise.
- No new dependency for a single phase without asking.
- No Faust, Cmajor, Halide, Mojo, Julia, or Elementary on the analysis path. The Sept 8 review rejected each for this workload (Faust's Rust backend is low-priority; Cmajor is GPLv3/commercial with a single-vendor bus-factor risk; Halide is 2D/stencil; Mojo and Julia have no wasm/Android target). Differentiable models (GOLF, DDSP articulatory) belong in C5 offline table generation, Phase 6 — not on device.
- No GPU compute for the 44-section solver. Report: trivial CPU work at 43 ms; dispatch latency and WebView WebGPU uncertainty cost more than they save.
- No third-party pitch crates (`pitch-detection` dormant since 2022). Reuse own C2 code.

---

## 7. First prompt for Claude Code (after Phase 0 gate)

> Read `PLAN_v3_pipeline.md` §2–§5 and `DECISIONS.md`. Implement Phase 1 only. Start with the acceptance tests in §5 Phase 1, then the `Stage` trait, `Pipeline` builder with type validation against the §2 table, preallocating `Runner`, and `Tap`. Wrap YIN, LPC, the grid inverse, and the Story tract as stages using their existing tests as contract tests — do not change their internals. Every numeric value goes in `pipeline.toml` or a `StageConfig`. Fail loud on any wiring error and print the adapter chain. Stop at the Phase 1 gate and report latency numbers.
