//! The Vocal Tract Lab atlas (Plan v3 D10, Phase 3): the reduced MRI model
//! and its lumen mesh, compiled in from `assets/vocal_tract_lab/` with
//! their digests checked, and the kernels the app runs on them, ported
//! from the decompiled 0.10.0 APK (`research/vocal-tract-lab-0.10.0/`).
//!
//! Everything here is model arithmetic with no stage plumbing; the stages
//! in `pipeline::stages::{posterior, mri_tract, mesh}` wrap it. The
//! model's own words about itself (`scientific_release_ready: false`, zero
//! independent acceptances, the surface-error miss) are carried into the
//! provenance record and the Evidence output unchanged.

pub mod articulators;
pub mod lumen;
pub mod reduced_model;

pub use reduced_model::{AbstainReason, MAX_MODES, ReducedModel, TemporalAtlasFilter};

use serde::{Deserialize, Serialize};

/// One compiled-in data file, as the provenance record lists it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataFileProvenance {
    pub name: String,
    pub model_id: String,
    pub sha256: String,
    pub scientific_release_ready: bool,
    pub independent_expert_acceptances: u32,
    pub note: String,
}

/// The data files every pipeline compiles in, with their digests as
/// verified at load. A file that fails to load is still listed, with the
/// failure as its note — provenance is not the place to hide one.
pub fn data_provenance() -> Vec<DataFileProvenance> {
    let model = match reduced_model::shared() {
        Ok(m) => DataFileProvenance {
            name: "assets/vocal_tract_lab/reduced_model.json".into(),
            model_id: m.model_id.clone(),
            sha256: reduced_model::JSON_SHA256.into(),
            scientific_release_ready: m.atlas.scientific_release_ready,
            independent_expert_acceptances: m.atlas.independent_expert_acceptances,
            note: format!(
                "{} · freeze {} · {} subjects · source_model {} · freeze_manifest {}",
                m.description,
                m.atlas.freeze_version,
                m.atlas.subjects,
                m.atlas.source_model_sha256,
                m.atlas.freeze_manifest_sha256
            ),
        },
        Err(e) => DataFileProvenance {
            name: "assets/vocal_tract_lab/reduced_model.json".into(),
            model_id: String::new(),
            sha256: reduced_model::JSON_SHA256.into(),
            scientific_release_ready: false,
            independent_expert_acceptances: 0,
            note: format!("failed to load: {e}"),
        },
    };
    let mesh = match lumen::shared() {
        Ok(l) => DataFileProvenance {
            name: "assets/vocal_tract_lab/tract_lumen_v2.bin".into(),
            model_id: l.model_id.into(),
            sha256: lumen::SHA256.into(),
            scientific_release_ready: false,
            independent_expert_acceptances: 0,
            note: format!(
                "{} · {} sections × {} angular · embedded digests {} / {}",
                l.provenance, l.sections, l.angular, l.source_sha256[0], l.source_sha256[1]
            ),
        },
        Err(e) => DataFileProvenance {
            name: "assets/vocal_tract_lab/tract_lumen_v2.bin".into(),
            model_id: String::new(),
            sha256: lumen::SHA256.into(),
            scientific_release_ready: false,
            independent_expert_acceptances: 0,
            note: format!("failed to load: {e}"),
        },
    };
    vec![model, mesh]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_data_files_load_and_are_listed_with_their_digests() {
        let files = data_provenance();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].model_id, "vt3d-frozen-mri-pca-v0.7.0");
        assert!(!files[0].scientific_release_ready);
        assert_eq!(files[0].independent_expert_acceptances, 0);
        assert!(!files[0].note.starts_with("failed"), "{}", files[0].note);
        assert!(!files[1].note.starts_with("failed"), "{}", files[1].note);
        // The JSON's renderer digest is the lumen file's digest.
        let m = reduced_model::shared().unwrap();
        assert_eq!(m.atlas.renderer_lumen_sha256, lumen::SHA256);
        assert_eq!(files[1].sha256, lumen::SHA256);
    }
}
