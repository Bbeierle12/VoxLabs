# Phases 6–8 gate — the VTL boundary, anatomy, the external workbench

Plan v3 §5 Phases 6–8 (`docs/PLAN_v3_completion.md` §9–11), under the
Pixel-only directive. Phase 6 is desktop-only by the plan and has no
VocalTractLab library in reach here; what ships is the boundary and the
Workbench mode. Phase 7 has no data in hand. Phase 8's gate is closed.

## Phase 6 — the GPL boundary and the Workbench

| Plan item | Delivered |
|---|---|
| `vox-tract-vtl` behind a feature flag | `crates/vox-tract-vtl/`: the VocalTractLab C API declarations (`vtlInitialize`, `vtlGetConstants`, `vtlTractToTube`, `vtlGetTransferFunction`) behind the `vtl` feature; `VtlInverseStage` (`inverse/vtl_fit`) and `VtlTractStage` (`tract/vtl_tube`) implementing the `Stage` trait; without the feature (the default) they refuse to `init` and say why. The crate is GPL-3.0-or-later by its own manifest; `vox-core` does not depend on it. |
| Production build has no VTL symbols and no GPL code | `scripts/check-no-vtl.sh`, run in CI after the tests: `cargo tree` on `vox-core` names no VTL crate; the release cdylib carries no `vtl*` / `VocalTractLab` symbol or string. Passes. |
| 19-D `TractParams` | Not done: `TractParams` carries four mode slots; mapping VTL's 19 tract parameters onto the wire (a fifth `modes` size, or a separate payload) is the design question the scaffold documents and leaves for when the library exists. |
| Formant fitting and tube geometry through the API | Not done: the fitting loop and `vtlTractToTube` → `AreaFunction` need `libVocalTractLabApi` and a `.speaker` file, neither of which is in this environment. |
| C5's Python generating inverse tables and PCA bases through `vox-harness` | Not done; nothing derived from VTL exists, so nothing needs freezing or provenance. |
| Workbench mode: every tap exposed, two backends side by side in one slot | `pipelines/workbench.toml`: the Story grid and the atlas posterior both invert the same formants, each drives its own tract model, the mesh reads the atlas pair, every stage is tapped. Loads and builds (`definition::mode_tests`); selectable on the phone's PIPELINE card like every other mode. |
| O1 written before anything derived ships | Answered from the files in hand (below); nothing derived ships. |

### O1 — the written answer

Read on 2026-09-14 from `Bbeierle12/Vocal-Tract-Labs` at commit
`f3f907a` (`corpus/registry.csv`, `corpus/acquisition_manifest.csv`,
`LICENSES.md`, `DATA_POLICY.md`, `docs/`):

- The corpus registry lists twelve resources, all `link-only`; none is a
  VocalTractLab `.speaker` file or anything derived from one. No entry
  mentions VocalTractLab, Birkholz or the GPL.
- Licences: one entry is CC BY 4.0 (FRIEDRICHS2026, EMA + audio + head
  meshes); every other entry is "verify … terms" and nothing is marked
  redistributable.
- Phase 3's data files (`assets/vocal_tract_lab/`) descend from the app's
  own frozen metric-MRI subject means and a CC BY 4.0 rtMRI label set,
  not from `.speaker` files (`assets/vocal_tract_lab/PROVENANCE.md`).

So: **as of today no data in VoxLabs or in the corpus registry is
GPL-encumbered through VocalTractLab, because nothing has been derived
from VTL.** O1 becomes live the day `vox-tract-vtl` produces an inverse
table or a PCA basis; that output is not to ship without a written
licence opinion, and the CI symbol check keeps the code boundary
regardless. This is a reading of the files, not a legal opinion.

## Phase 7 — anatomy

"As data allows": it does not, yet. `TractGeometry` exists (Phase 3);
the corpus registry's anatomy resources are all link-only with terms to
verify, and none is in this environment. No registration or shape-model
stage is written against data that is not here. The first candidate when
data arrives: FRIEDRICHS2026 (CC BY 4.0, 3D head meshes, N = 18
anatomical subset) for the head-frame registration of the lumen mesh —
recorded in `DECISIONS.md` as the entry point, not started.

The surface-error target the plan wants "in config, reported by
validation" is there already: `[validation]` `surface_error_target_mm` /
`surface_error_reported_mm`, on the Evidence output since Phase 3.

## Phase 8 — external workbench

Gate to start: two people who are not Brandon asking to run their own
recordings, or a month of Brandon reaching for the Workbench over
Praat/Python. Neither has happened. Recorded closed; nothing scheduled.

## Gate

| Check | Result |
|---|---|
| `scripts/check-no-vtl.sh` | pass (no VTL dependency, no VTL symbols in `libvox_core.so`) |
| `vox-tract-vtl` builds and its stages refuse to init without the feature | pass (`crates/vox-tract-vtl` tests) |
| `workbench` mode loads and builds | pass (`definition::mode_tests`) |
| O1 written | above |
