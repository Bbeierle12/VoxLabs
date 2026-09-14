# Vocal Tract Lab 0.10.0 — decompiled reference (not source)

This folder is **decompiled output**, not the app's source. Plan v3
completion-plan call 6 (DECISIONS.md): Phase 3 proceeds from the shipped
APK because the Kotlin repository was not available; the source is still
wanted for fine-grained parity (O4).

How it was produced (2026-09-14):

1. `vocal-tract-lab-0.10.0-shared-model.apk` (Brandon's build, package
   `org.vocaltract.pixel`) unzipped: `classes.dex`, `res/raw/`
   (`reduced_model.json`, `tract_lumen_v2.bin` → `assets/vocal_tract_lab/`),
   `assets/anatomy/` (the Evidence/bench web view; `index.html` kept here as
   `anatomy-index.html` for the Evidence-tab wording and the surface-error
   figure).
2. `jadx` on `classes.dex`; only `org/vocaltract/pixel/*.java` is kept
   (83 files). Kotlin metadata annotations, synthetic accessors and jadx's
   control-flow reconstruction are as the tool emitted them — read the
   arithmetic, not the structure.

What `vox-core` ports from it (Phase 3), each with the app's formulas as
its contract test:

| Decompiled class | Port |
|---|---|
| `ReducedModelAsset` (validation, `inferArea`, `areaFromCoefficients`) | `src/atlas/reduced_model.rs` |
| `TemporalAtlasFilter`, `TemporalAtlasEstimate` | `src/atlas/reduced_model.rs::TemporalAtlasFilter` |
| `FrameAnalyzer` (evidence confidence, f0 penalty, relative area std) | `src/pipeline/stages/posterior.rs` |
| `TractLumenAsset` (+`Kt` helpers: header, CRC-32, `morph`, `ringAreaCm2`) | `src/atlas/lumen.rs` |
| `TractMeshCpu` (`resampleArea`, `computeVertexNormals`, `expandUncertainty`) | `src/atlas/lumen.rs` |
| `ArticulatorPosterior.infer` | `src/atlas/articulators.rs` |
| `AdminConsoleActivity`, `DiagnosticsRuntime`, `DiagnosticStore`, … | `src/diagnostics/` (ported earlier) |

Not ported: `SharedTractModel.fit` / `TubeGrid` (the app's own 1-D tube
solver and its iterative fit; VoxLabs' chain-matrix solver in `tract.rs`
is the forward model here — see `docs/phase3-gate.md` for what that means
for parity), the WebView bench, and the Kotlin audio capture (cpal on the
VoxLabs side).
