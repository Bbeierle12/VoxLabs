# Vocal Tract Lab data files — provenance

Both files are copied byte-for-byte from the shipped Vocal Tract Lab
0.10.0 APK (`res/raw/`, package `org.vocaltract.pixel`, Brandon's Kotlin
app; see `research/vocal-tract-lab-0.10.0/README.md` for how they were
extracted). `vox-core` compiles them in (`src/atlas/`) and refuses to
start a pipeline on them unless their SHA-256 matches the values below.

| File | SHA-256 | What it is |
|---|---|---|
| `reduced_model.json` | `838e719085a85b577394190cbf26a2525af56de34eeba774ac32c37734e55c2e` | Schema 2 reduced model `vt3d-frozen-mri-pca-v0.7.0`: 32 area sections at normalized positions, 4 log-area PCA modes, reference formants, the ridge-regularized formant→mode map, coefficient limits, uncertainty block, atlas block |
| `tract_lumen_v2.bin` | `8ca7be68088afbe93f3327f52db1f34a790217e8af130cb47c39acecbd472006` | Lumen surface mesh (`VTLUMN2`, schema 2): 32 sections × 24 angular samples, centerline and ring offsets in mm, reference areas, triangle indices, CRC-32 |

The JSON's own atlas block, carried unchanged (the Evidence output shows
it):

- `model_id`: `vt3d-frozen-mri-pca-v0.7.0`, freeze version 0.6.3
- `source_model_sha256`: `1ae2d6aa55982295e632105c4b42ee27d80f132880233054bd2773a3e2d95e6f`
- `renderer_lumen_sha256`: `8ca7be68088afbe93f3327f52db1f34a790217e8af130cb47c39acecbd472006` (equals the lumen file's digest)
- `freeze_manifest_sha256`: `6b9212fe185e07258e436ccadd6442451721688a43e0cb9e3ba9334e703fc527`
- subjects: 5 (`frozen_subject_mean_P1/P4/P5/P7/P10`); explained variance 0.470 / 0.239 / 0.181 / 0.111
- `independent_expert_acceptances`: 0
- `scientific_release_ready`: **false**
- boundary: "Engineering review is resolved and the baseline is reproducibly
  frozen. No record has independent expert acceptance, population/error
  gates still fail, and this is not a clinical or singer-population atlas."
- uncertainty: "Audio-conditioned estimate; anatomy is not observed."
- acoustic inverse: "Locally identifies area-function modes only; full 3-D
  acoustic inversion remains many-to-one." (ridge 6.2, condition number 29.7)

Held-out surface error, as the app's own Evidence tab states it
(`anatomy/index.html` in the APK): **median 4.781 mm · target ≤ 4 mm · not
met**. `pipeline.toml` `[validation]` carries both numbers so the Evidence
output reports the miss rather than hiding it.

What the source MRI data is: the app's MRI-label reference (`reference.js`)
cites Ruthven, Peplinski and Miquel (2023), Zenodo record 10046815,
CC BY 4.0 — 2-D real-time MRI labels, supine English counting, not singing,
`metric_geometry: false`. The frozen subject means behind the reduced model
are the app's own 0.6.3 freeze; no VocalTractLab (Birkholz) `.speaker`
data is involved (see DECISIONS.md, O1 note).

These are published data tables in the sense of `src/config/mod.rs`: their
numbers are the model, not settings, and are not mirrored in
`pipeline.toml`.
