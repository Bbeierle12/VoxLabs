//! `vox-tract-vtl` — VocalTractLab (Birkholz, GPL-3.0) as `Inverse`
//! backend #3 and `TractModel` backend #3 for the desktop Workbench
//! (Plan v3 D2/D3, Phase 6). The GPL boundary is the crate boundary:
//! `vox-core` never depends on this crate, the APK never links it, and
//! `scripts/check-no-vtl.sh` (CI) proves the production library carries
//! no VTL symbol. Everything derived for production (inverse tables, PCA
//! bases) is frozen as data files with provenance, and only after O1 has
//! its written answer (`DECISIONS.md`).
//!
//! What is here today is the scaffold Phase 6 builds on: the C API
//! declarations of `VocalTractLabApi` behind the `vtl` feature, the
//! 19-parameter `TractParams` mapping, and the two stage types. Without
//! the feature the stages compile, register nowhere, and refuse to
//! `init` with a message that says why — no silent stub.

use vox_core::config::PipelineParams;
use vox_core::pipeline::stage::{Stage, StageError, StreamFormat};
use vox_core::pipeline::types::{AreaFunction, F0Track, FormantTrack, TractParams};

/// VocalTractLab's vocal-tract parameter count (the `.speaker` file's
/// tract parameters: HX, HY, JX, JA, LP, LD, VS, VO, TCX, TCY, TTX, TTY,
/// TBX, TBY, TRX, TRY, TS1..TS4 in VTL 2.3 — 19 when TS4 is omitted, as
/// the plan counts them). The mapping onto `TractParams::modes` (four
/// slots) is the Phase 6 design question this scaffold leaves open.
pub const VTL_TRACT_PARAMS: usize = 19;

/// The subset of the VocalTractLab C API the backends need, as declared in
/// `VocalTractLabApi.h`. Linked only with the `vtl` feature.
#[cfg(feature = "vtl")]
pub mod ffi {
    use std::os::raw::{c_char, c_double, c_int};

    #[link(name = "VocalTractLabApi")]
    unsafe extern "C" {
        pub fn vtlInitialize(speaker_file_name: *const c_char) -> c_int;
        pub fn vtlClose() -> c_int;
        pub fn vtlGetConstants(
            audio_sampling_rate: *mut c_int,
            num_tube_sections: *mut c_int,
            num_vocal_tract_params: *mut c_int,
            num_glottis_params: *mut c_int,
            num_audio_samples_per_tract_state: *mut c_int,
            internal_sampling_rate: *mut c_double,
        ) -> c_int;
        pub fn vtlTractToTube(
            tract_params: *const c_double,
            tube_length_cm: *mut c_double,
            tube_area_cm2: *mut c_double,
            tube_articulator: *mut c_int,
            incisor_pos_cm: *mut c_double,
            tongue_tip_side_elevation: *mut c_double,
            velum_opening_cm2: *mut c_double,
        ) -> c_int;
        pub fn vtlGetTransferFunction(
            tract_params: *const c_double,
            num_spectrum_samples: c_int,
            opts: *const std::ffi::c_void,
            magnitude: *mut c_double,
            phase_rad: *mut c_double,
        ) -> c_int;
    }
}

fn not_linked(what: &str) -> StageError {
    StageError::Init(format!(
        "{what}: vox-tract-vtl was built without the `vtl` feature (VocalTractLab, GPL, \
         is never linked into the production build); build the Workbench with \
         `--features vox-tract-vtl/vtl` and a libVocalTractLabApi on the library path"
    ))
}

/// `Inverse` backend #3: VocalTractLab formant fitting.
#[derive(Debug)]
pub struct VtlInverseStage;

impl Stage for VtlInverseStage {
    type In<'a> = (&'a F0Track, &'a FormantTrack);
    type Out = TractParams;
    const NAME: &'static str = "inverse";
    const BACKEND: &'static str = "vtl_fit";
    const VERSION: &'static str = "0.1.0";

    fn init(_params: &PipelineParams, _fmt: &StreamFormat) -> Result<Self, StageError> {
        #[cfg(feature = "vtl")]
        {
            return Err(StageError::Init(
                "vtl_fit: the VocalTractLab fitting loop is Phase 6 work; the API is linked but \
                 the fit is not implemented"
                    .into(),
            ));
        }
        #[cfg(not(feature = "vtl"))]
        Err(not_linked("vtl_fit"))
    }

    fn process(
        &mut self,
        _input: (&F0Track, &FormantTrack),
        _out: &mut TractParams,
    ) -> Result<(), StageError> {
        Err(not_linked("vtl_fit"))
    }
}

/// `TractModel` backend #3: VocalTractLab's tube geometry.
#[derive(Debug)]
pub struct VtlTractStage;

impl Stage for VtlTractStage {
    type In<'a> = &'a TractParams;
    type Out = AreaFunction;
    const NAME: &'static str = "tract";
    const BACKEND: &'static str = "vtl_tube";
    const VERSION: &'static str = "0.1.0";

    fn init(_params: &PipelineParams, _fmt: &StreamFormat) -> Result<Self, StageError> {
        #[cfg(feature = "vtl")]
        {
            return Err(StageError::Init(
                "vtl_tube: vtlTractToTube → AreaFunction is Phase 6 work; the API is linked but \
                 the mapping is not implemented"
                    .into(),
            ));
        }
        #[cfg(not(feature = "vtl"))]
        Err(not_linked("vtl_tube"))
    }

    fn process(&mut self, _p: &TractParams, _out: &mut AreaFunction) -> Result<(), StageError> {
        Err(not_linked("vtl_tube"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_the_feature_the_stages_refuse_to_init_and_say_why() {
        let fmt = StreamFormat {
            sample_rate_hz: 48_000.0,
            frame_samples: 2048,
            hop: 1024,
        };
        let e = VtlInverseStage::init(&PipelineParams::DEFAULT, &fmt).unwrap_err();
        assert!(e.to_string().contains("vtl"), "{e}");
        let e = VtlTractStage::init(&PipelineParams::DEFAULT, &fmt).unwrap_err();
        assert!(e.to_string().contains("GPL"), "{e}");
        assert_eq!(VTL_TRACT_PARAMS, 19);
    }
}
