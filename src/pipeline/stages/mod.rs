//! The wrapped kernels. Each is a `Stage` around an existing, tested
//! function — YIN, the SNR/hum voicing gates, LPC/Levinson + root-solve,
//! Goertzel harmonics, the voice-quality metrics, the f0 contour, the
//! (q1, q2) grid inverse, the Story area function, the STFT — with that
//! function's tests as its contract.
//! Internals are untouched (§3: wrap, don't rewrite).
//!
//! One consequence, stated rather than hidden: the wrapped functions read
//! their parameters from `config::*::DEFAULT` internally, so a mode file's
//! `[params]` override of a section a kernel reads would silently not
//! apply. Each stage's `init` therefore refuses such an override with an
//! error naming the section (fail loud), except for the keys the wrapper
//! itself passes through (`tract.grid_n`).

pub mod contour;
pub mod harmonics;
pub mod inverse;
pub mod lpc;
pub mod metrics;
pub mod stft;
pub mod tract;
pub mod voicing;
pub mod yin;

use super::stage::{StageDescriptor, StageError, describe};

/// Every backend the builder can instantiate.
pub fn registry() -> &'static [StageDescriptor] {
    static REGISTRY: &[StageDescriptor] = &[
        describe::<yin::YinStage>(),
        describe::<voicing::VoicingStage>(),
        describe::<lpc::LpcStage>(),
        describe::<harmonics::HarmonicsStage>(),
        describe::<metrics::MetricsStage>(),
        describe::<contour::ContourStage>(),
        describe::<inverse::GridInverseStage>(),
        describe::<tract::StoryTractStage>(),
        describe::<stft::StftStage>(),
    ];
    REGISTRY
}

/// Refuses a `[params]` override of a section the wrapped kernel reads
/// from its compiled default.
pub(crate) fn require_default<T: PartialEq + std::fmt::Debug>(
    section: &str,
    stage: &str,
    given: &T,
    default: &T,
) -> Result<(), StageError> {
    if given == default {
        Ok(())
    } else {
        Err(StageError::Init(format!(
            "stage `{stage}` cannot honor a [params.{section}] override: its kernel reads \
             {section}'s compiled defaults internally (Phase 2 threads config through); \
             remove the override or change pipeline.toml"
        )))
    }
}
