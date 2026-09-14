# Phase 3 gate — Vocal Tract Lab backends from the decompile

Plan v3 §5 Phase 3 (`docs/PLAN_v3_completion.md` §4). What was ported,
what the contract tests prove, what Experiment 3 measured, and what the
gate says. Short version: the port is exact to the decompiled formulas
and runs on the phone; the posterior inverse does **not** meet the
Experiment 3 accuracy band under VoxLabs' forward model; parity with the
app itself (bundle against bundle) still needs the Pixel.

## What was built

| Piece | Where | Ported from (research/vocal-tract-lab-0.10.0/) |
|---|---|---|
| Reduced model loader, validation, `infer_coefficients`, `area_from_coefficients` | `src/atlas/reduced_model.rs` | `ReducedModelAsset` |
| Temporal atlas filter (follow / decay / abstain, reasons) | `src/atlas/reduced_model.rs::TemporalAtlasFilter` | `TemporalAtlasFilter` |
| Lumen mesh: `VTLUMN2` header, CRC-32, ring-area check, `morph`, normals, uncertainty envelope, area resampling | `src/atlas/lumen.rs` | `TractLumenAsset`, `TractMeshCpu` |
| Articulator readout (jaw, lips, tongue, pharynx, epilarynx) | `src/atlas/articulators.rs` | `ArticulatorPosterior.infer` |
| `inverse/posterior_pca4`: evidence score, f0 penalty, filter, relative area std | `src/pipeline/stages/posterior.rs` | `FrameAnalyzer` |
| `tract/mri_pca4`: 32-section area function, tract length from the lumen centerline | `src/pipeline/stages/mri_tract.rs` | `ReducedModelAsset.areaFromCoefficients` |
| `mesh/lumen_v2`: `TractGeometry` (vertices, normals, uncertainty mesh, triangles, frame note) | `src/pipeline/stages/mesh.rs` | `TractMeshCpu.prepare` |
| Data files with digests checked at load | `assets/vocal_tract_lab/` (+ `PROVENANCE.md`) | `res/raw/` |
| Mode file | `pipelines/atlas.toml` (yin → voicing → lpc → harmonics → metrics → contour → inverse → tract → mesh → stft) | — |
| Config | `[posterior]`, `[articulators]`, `[validation]` in `pipeline.toml`; `mode_sd`, `geometry_mm` in `[tolerance]` | the app's literals |
| Types | `TractParams` (model, four `modes`, confidence, abstained, reason), `AreaFunction.model`/32 sections, `TractGeometry`, `BasisId::MriAtlasMean`, `FormantTrack.f4` | Plan §2 table |
| Provenance / Evidence | `ProvenanceRecord.data_files`; Evidence rows "Atlas data files…", "Atlas held-out surface error", "Independent expert acceptances", "Scientific release readiness" | the JSON's atlas block; the app's Evidence tab |
| Harness | `voxlab gen-inverse-fixtures <out> [--n] [--seed]` (Experiment 3), `voxlab run atlas <audio>` / `validate` | — |
| On-device | Engineering Console → Run self-test: `contract.atlas_data_files_verify`, `contract.posterior_matches_reference`, `contract.lumen_mesh_morphs`, `contract.atlas_mode_runs_a_vowel` | — |

Not ported: `SharedTractModel.fit` and `TubeGrid` (the app's own tube
solver and iterative refinement; the app only runs them on frames with
three tracked formants and falls back to `inferArea` otherwise), the
WebView bench, the Kotlin audio capture.

Two deviations, stated in the stage docs rather than hidden:

- The app's *harmonicity* is its own 0..1 measure. VoxLabs maps HNR
  onto 0..1 over `posterior.harmonicity_hnr_span_db` (20 dB).
- The app feeds its tracked formants (up to four) to the map. VoxLabs'
  `FormantTrack` carried three; it now also carries `f4` (the fourth
  in-band LPC pole when the frame resolves one — `math::formant_candidates`,
  the same list the three come from). The posterior uses four when it has
  them, three otherwise. See "Why F4 matters" below.

## Contract checks (host and phone)

| Check | What it proves | Host |
|---|---|---|
| `atlas::reduced_model::tests` (5) | schema 2, 4 × 32, the JSON's numbers; `inferArea` = clamped linear map (zero at the reference formants, row sums at +100 Hz, ±2 clamp); `areaFromCoefficients` = exp(ln mean + Σ c·mode) with the [0.2, 5]·mean clamp; the filter's follow rate α = clamp(0.38·e + 0.12, 0.12, 0.46), confidence 0.35-follow, 0.9 / 0.82 decay, reason strings; a tampered JSON is refused | pass |
| `atlas::lumen::tests` (4) | header, payload length, CRC-32, the app's ring-area consistency check (< 0.2 %); morph at the reference areas reproduces the stored mesh exactly, √2 scaling at doubled area, unit normals, the √(1 + σ) envelope; resampling; a flipped byte fails the CRC | pass |
| `atlas::articulators::tests` | the mean shape reads zero; a doubled lip region reads ln 2 | pass |
| `stages::posterior::tests` (3) | the evidence formula hand-computed (weights, f0 penalty, floor); one strong frame follows the map at α; unvoiced / rejected → `unvoiced`; weak voiced → `insufficient_evidence` | pass |
| `stages::mri_tract::tests` | stage areas = model synthesis; 32 sections; diameters; Story params refused | pass |
| `stages::mesh::tests` | vertices = lumen morphed to the resampled mean; envelope; a Story area function meshes too | pass |
| `vox-validation` `atlas_mode_round_trips_with_geometry_and_reports_the_atlas_statements` | record → rebuild → re-run within the bands for TractParams, AreaFunction **and TractGeometry**; the provenance record lists both data files with `scientific_release_ready: false`; the Evidence output shows the surface-error miss; the posterior produced evidence frames on the vowel | pass |
| `pipeline::contract` `atlas_*` / `posterior_*` / `lumen_*` (4) | the same, as the Console's self-test on the Pixel | pass on host; **to be read off the phone** |

`cargo test --workspace`: 205 library, 6 harness, 3 validation.
`cargo clippy --workspace --all-targets -- -D warnings` clean. Android
and wasm library type-checks pass.

## Experiment 3 (`voxlab gen-inverse-fixtures`, seed 20260914, 1000 pairs per backend, x86_64 release)

Gate as written: 95th-percentile worst-of-F1..F3 round-trip error < 5 %
or < 50 Hz, and < 5 ms per frame (`[validation]`).

| Backend | Evaluated | rel p50 | rel p95 | abs p50 | abs p95 | F4 p95 | param RMS | latency p95 | Accuracy | Latency |
|---|---|---|---|---|---|---|---|---|---|---|
| `grid_story` (F1–F2, adult-male basis, 41 × 41) | 992 (8 with no third resonance in band) | 0.5 % | 1.9 % | 4 Hz | 12 Hz | — | 0.21 (q units) | 0.003 ms | **met** | met |
| `posterior_pca4`, coefficients uniform over ±2 SD | 1000 | 20.7 % | 42.4 % | 265 Hz | 732 Hz | 1407 Hz | 1.54 SD | 0.0002 ms | **not met** | met |
| `posterior_pca4`, coefficients within ±0.5 SD | 1000 | 4.0 % | 9.6 % | 110 Hz | 184 Hz | 250 Hz | 0.71 SD | 0.0003 ms | **not met** | met |

The fixture files (`inverse_fixtures_<backend>.jsonl`) and
`inverse_eval.json` (with the bands, the seed, the data-file digests and
the Jacobian comparison below) are what the command writes; they are
reproducible from the seed and are not checked in.

Forward model for the posterior rows: the atlas area function resampled
to the solver's 44 sections, the lumen centerline as the tract length,
VoxLabs' lossless chain-matrix solver (`tract::resonances_up_to`,
ceiling 6 kHz so F4 exists). This is **not** the app's `TubeGrid`, which
the app linearized to build its map. How far apart the two forward
models are, at the atlas mean (Hz per SD, central differences, 0.1 SD):

| | mode 1 | mode 2 | mode 3 | mode 4 |
|---|---|---|---|---|
| F1 app / VoxLabs | −100 / −100 | −110 / −114 | −60 / −60 | 10 / 17 |
| F2 app / VoxLabs | 270 / 220 | 40 / 65 | −20 / −16 | −140 / −158 |
| F3 app / VoxLabs | 230 / 189 | −90 / −54 | −120 / −157 | −130 / −127 |
| F4 app / VoxLabs | 70 / 83 | −30 / −83 | −100 / −42 | −140 / −156 |

Relative Frobenius difference over F1..F3: 0.19. The two solvers agree
on the local sensitivities to within about a fifth; the map's failure is
not mainly a forward-model disagreement.

### Why F4 matters

`formant_to_mode` in the JSON is, to three decimals, the exact inverse of
the JSON's own 4 × 4 `jacobian_hz_per_sd` (`M·J ≈ I`; largest
off-diagonal 0.009). Drop the F4 column and row and `M[:, :3]·J[:3, :]`
is nowhere near identity (diagonal 1.28, 1.26, −0.04, 0.47; off-diagonals
up to 1.4). The app's map needs four formants; with three it is a
different, much worse map. That is why `FormantTrack.f4` was added in
this phase: the ±0.5 SD row improved from a 34 % to a 9.6 % 95th
percentile once F4 was fed in. What remains is the map's own nature: a
local linearization the JSON itself describes as "Locally identifies
area-function modes only; full 3-D acoustic inversion remains
many-to-one" (condition number 29.7, ridge 6.2). Coefficients sampled
over the full ±2 SD box leave its linear regime and hit the clamps.

### Verdict

- `grid_story` meets the Experiment 3 bands.
- `posterior_pca4` meets the latency band by four orders of magnitude and
  does not meet the accuracy band, in either regime. Per the plan's
  exit clause the fallback decision is recorded in `DECISIONS.md`
  (Phase 3 status), not worked around here: the posterior stays as the
  **parity target** for the bundle comparison against the app (it is the
  app's algorithm), and is not offered as an accuracy-grade inverse;
  `live_model` keeps the Story grid.

## Parity with the app (Plan §4.2)

Not run yet — it needs the Pixel:

1. Install Vocal Tract Lab 0.10.0 alongside this build.
2. Play the same recording into both (the app's `WavPcmReplay` layer or
   the same loudspeaker take), export both diagnostic bundles.
3. Compare the app's 500 ms `derived_state_sample` events (f0, tract
   confidence, abstention reason, relative area std) with VoxLabs'
   `derived_state_sample` events from the `atlas` mode; the D5 bands for
   the coarse fields are `[tolerance]` `confidence` and `f0_hz`.

Until that comparison exists, D10's "Kotlin DSP retires after
tolerance-band parity" is not met and is not marked met.

## Frame of `TractGeometry`

The plan's type table says "VTL frame canonical". The lumen mesh is in
the renderer's millimetre frame as stored; nothing in the APK records
a transform to any other frame. `TractGeometry.frame` says so
(`atlas::lumen::FRAME`), and no transform was invented.

## O1 and O4

Recorded in `DECISIONS.md` (Phase 3 status). In one line each: the
Phase 3 data files derive from the app's own frozen metric-MRI subject
means and a CC BY 4.0 2-D rtMRI label set (Ruthven, Peplinski and
Miquel 2023), not from VocalTractLab `.speaker` data — the answer is
from the files in hand, the `Vocal-Tract-Labs` corpus registry was not
available to read; the port proceeded from the decompiled APK and the
source is still wanted for the fine-grained parity gate.

## Not done in this phase

- Bundle-vs-bundle parity on the phone (above).
- `SharedTractModel.fit` / `TubeGrid` (the app's refinement path).
- The frozen atlas's own gates (surface error, expert acceptance) are
  reported, not improved — they are the atlas's, not this code's.
- The Pixel's live screen still runs `live_model` (Story). Selecting the
  `atlas` mode on the phone is Phase 5a's shell work.
