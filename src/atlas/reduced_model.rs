//! The reduced model (`reduced_model.json`, schema 2,
//! `vt3d-frozen-mri-pca-v0.7.0`) and the two kernels Vocal Tract Lab runs
//! on it, ported from the decompiled `ReducedModelAsset` and
//! `TemporalAtlasFilter`: the formant→coefficient inference, the log-area
//! synthesis, and the temporal filter with its abstention. The validation
//! rules are the app's own constructor checks. The model is 16 kHz /
//! 1024 / 512 in the app, but the inverse consumes formants only, so it
//! is rate-agnostic here.

use std::sync::OnceLock;

use serde::Deserialize;

use crate::config::PosteriorConfig;
use crate::hash::sha256_hex;

/// The most modes a `TractParams` carries (the atlas has four).
pub const MAX_MODES: usize = 4;
pub const JSON: &str = include_str!("../../assets/vocal_tract_lab/reduced_model.json");
pub const JSON_SHA256: &str = "838e719085a85b577394190cbf26a2525af56de34eeba774ac32c37734e55c2e";
/// The app's default when the JSON has no uncertainty block.
const DEFAULT_ABSTENTION_THRESHOLD: f32 = 0.22;
/// The app's default coefficient limits, standard deviations.
const DEFAULT_LIMITS_SD: [f32; 2] = [-2.0, 2.0];

#[derive(Clone, Debug, Default, PartialEq, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Atlas {
    pub source_model_sha256: String,
    pub freeze_manifest_sha256: String,
    pub renderer_lumen_sha256: String,
    pub freeze_version: String,
    pub subjects: u32,
    pub subject_ids: Vec<String>,
    pub components: u32,
    pub explained_variance_ratio: Vec<f32>,
    pub independent_expert_acceptances: u32,
    pub scientific_release_ready: bool,
    pub boundary: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Uncertainty {
    pub abstention_confidence_threshold: f32,
    pub maximum_relative_area_std: f32,
    pub minimum_relative_area_std: f32,
    pub interpretation: String,
}

impl Default for Uncertainty {
    fn default() -> Self {
        Self {
            abstention_confidence_threshold: DEFAULT_ABSTENTION_THRESHOLD,
            maximum_relative_area_std: 0.0,
            minimum_relative_area_std: 0.0,
            interpretation: String::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AcousticInverse {
    pub method: String,
    pub ridge: f32,
    pub condition_number: f32,
    pub jacobian_hz_per_sd: Vec<Vec<f32>>,
    pub interpretation: String,
}

#[derive(Deserialize)]
struct Raw {
    schema_version: u32,
    model_id: String,
    sample_rate_hz: u32,
    frame_size: usize,
    hop_size: Option<usize>,
    section_position: Vec<f32>,
    area_mean_cm2: Vec<f32>,
    area_modes: Vec<Vec<f32>>,
    reference_formants_hz: Vec<f32>,
    coefficient_scale_hz: Vec<f32>,
    #[serde(default)]
    formant_to_mode: Vec<Vec<f32>>,
    coefficient_limits_sd: Option<Vec<[f32; 2]>>,
    mode_labels: Option<Vec<String>>,
    #[serde(default)]
    atlas: Atlas,
    #[serde(default)]
    uncertainty: Uncertainty,
    #[serde(default)]
    acoustic_inverse: AcousticInverse,
    #[serde(default)]
    description: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReducedModel {
    pub schema_version: u32,
    pub model_id: String,
    pub sample_rate_hz: u32,
    pub frame_size: usize,
    pub hop_size: usize,
    /// Normalized glottis-to-lips position of each section, 0..1, increasing.
    pub section_position: Vec<f32>,
    pub area_mean_cm2: Vec<f32>,
    /// `area_modes[mode][section]`, log-area units per standard deviation.
    pub area_modes: Vec<Vec<f32>>,
    pub reference_formants_hz: Vec<f32>,
    pub coefficient_scale_hz: Vec<f32>,
    /// `formant_to_mode[mode][formant]`, SD per Hz (schema 2).
    pub formant_to_mode: Vec<Vec<f32>>,
    pub coefficient_limits_sd: Vec<[f32; 2]>,
    pub mode_labels: Vec<String>,
    pub atlas: Atlas,
    pub uncertainty: Uncertainty,
    pub acoustic_inverse: AcousticInverse,
    pub description: String,
}

impl ReducedModel {
    pub fn parse(text: &str) -> Result<Self, String> {
        let r: Raw = serde_json::from_str(text).map_err(|e| format!("reduced_model.json: {e}"))?;
        let n_modes = r.area_modes.len();
        let m = Self {
            schema_version: r.schema_version,
            model_id: r.model_id,
            sample_rate_hz: r.sample_rate_hz,
            frame_size: r.frame_size,
            hop_size: r.hop_size.unwrap_or(r.frame_size),
            section_position: r.section_position,
            area_mean_cm2: r.area_mean_cm2,
            area_modes: r.area_modes,
            reference_formants_hz: r.reference_formants_hz,
            coefficient_scale_hz: r.coefficient_scale_hz,
            formant_to_mode: r.formant_to_mode,
            coefficient_limits_sd: r
                .coefficient_limits_sd
                .filter(|l| !l.is_empty())
                .unwrap_or_else(|| vec![DEFAULT_LIMITS_SD; n_modes]),
            mode_labels: r
                .mode_labels
                .filter(|l| !l.is_empty())
                .unwrap_or_else(|| (1..=n_modes).map(|i| format!("Reduced mode {i}")).collect()),
            atlas: r.atlas,
            uncertainty: r.uncertainty,
            acoustic_inverse: r.acoustic_inverse,
            description: r.description,
        };
        m.validate()?;
        Ok(m)
    }

    /// The app's constructor requirements, each named.
    fn validate(&self) -> Result<(), String> {
        let n = self.section_position.len();
        let modes = self.area_modes.len();
        let req = |ok: bool, what: &str| -> Result<(), String> {
            if ok {
                Ok(())
            } else {
                Err(format!("reduced model: {what}"))
            }
        };
        req(
            self.schema_version == 1 || self.schema_version == 2,
            &format!("unsupported schema {}", self.schema_version),
        )?;
        req(
            (1..=self.frame_size).contains(&self.hop_size),
            "hop_size must be 1..=frame_size",
        )?;
        req(n >= 16, "fewer than 16 sections")?;
        req(self.area_mean_cm2.len() == n, "area_mean_cm2 length")?;
        req(
            modes == self.reference_formants_hz.len(),
            "one mode per reference formant",
        )?;
        req(
            self.reference_formants_hz.len() == self.coefficient_scale_hz.len(),
            "coefficient_scale_hz length",
        )?;
        req(
            self.area_modes.iter().all(|m| m.len() == n),
            "area_modes row length",
        )?;
        req(
            self.area_mean_cm2.iter().all(|&a| a.is_finite() && a > 0.0),
            "area_mean_cm2 must be finite and positive",
        )?;
        req(
            self.section_position.windows(2).all(|w| w[1] > w[0]),
            "section_position must increase",
        )?;
        req(
            self.coefficient_limits_sd.len() == modes,
            "coefficient_limits_sd length",
        )?;
        req(
            self.coefficient_limits_sd.iter().all(|l| l[0] < l[1]),
            "coefficient_limits_sd must be [lo, hi] with lo < hi",
        )?;
        if self.schema_version == 2 {
            req(self.formant_to_mode.len() == modes, "formant_to_mode rows")?;
            req(
                self.formant_to_mode
                    .iter()
                    .all(|r| r.len() == self.reference_formants_hz.len()),
                "formant_to_mode columns",
            )?;
            req(self.mode_labels.len() == modes, "mode_labels length")?;
        }
        req(modes <= MAX_MODES, &format!("more than {MAX_MODES} modes"))?;
        Ok(())
    }

    pub fn n_modes(&self) -> usize {
        self.area_modes.len()
    }

    pub fn n_sections(&self) -> usize {
        self.section_position.len()
    }

    pub fn limits(&self) -> [[f32; 2]; MAX_MODES] {
        let mut l = [DEFAULT_LIMITS_SD; MAX_MODES];
        for (i, lim) in self.coefficient_limits_sd.iter().enumerate() {
            l[i] = *lim;
        }
        l
    }

    /// `ReducedModelAsset.inferArea`'s coefficient half: the clamped
    /// linear map from measured formants (as many as given, up to the
    /// reference count) to mode coefficients. Modes beyond the model's are
    /// left at 0.
    pub fn infer_coefficients(&self, formants_hz: &[f32], out: &mut [f32; MAX_MODES]) {
        *out = [0.0; MAX_MODES];
        let used = formants_hz.len().min(self.reference_formants_hz.len());
        if self.schema_version == 2 {
            for (i, row) in self.formant_to_mode.iter().enumerate() {
                let c: f32 = row
                    .iter()
                    .zip(formants_hz)
                    .zip(&self.reference_formants_hz)
                    .take(used)
                    .map(|((w, f), r)| w * (f - r))
                    .sum();
                let [lo, hi] = self.coefficient_limits_sd[i];
                out[i] = c.clamp(lo, hi);
            }
        } else {
            for i in 0..used.min(self.n_modes()) {
                let c =
                    (formants_hz[i] - self.reference_formants_hz[i]) / self.coefficient_scale_hz[i];
                let [lo, hi] = self.coefficient_limits_sd[i];
                out[i] = c.clamp(lo, hi);
            }
        }
    }

    /// `ReducedModelAsset.areaFromCoefficients`: a_k = exp(ln mean_k +
    /// Σ_i clamp(c_i) · mode_i[k]), clamped to `[floor, ceiling] · mean_k`.
    /// Writes the first `n_sections()` entries of `out`.
    pub fn area_from_coefficients(
        &self,
        coefficients: &[f32],
        cfg: &PosteriorConfig,
        out: &mut [f32],
    ) -> Result<(), String> {
        let n = self.n_sections();
        if coefficients.len() < self.n_modes() {
            return Err(format!(
                "area_from_coefficients: {} coefficients for {} modes",
                coefficients.len(),
                self.n_modes()
            ));
        }
        if out.len() < n {
            return Err(format!(
                "area_from_coefficients: {} slots for {n} sections",
                out.len()
            ));
        }
        for k in 0..n {
            let mean = self.area_mean_cm2[k];
            let mut log = mean.ln();
            for (i, mode) in self.area_modes.iter().enumerate() {
                let [lo, hi] = self.coefficient_limits_sd[i];
                log += coefficients[i].clamp(lo, hi) * mode[k];
            }
            out[k] = log
                .exp()
                .clamp(cfg.area_floor_ratio * mean, cfg.area_ceiling_ratio * mean);
        }
        Ok(())
    }
}

/// Why the filter abstained on a frame (the app's `reason` strings).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AbstainReason {
    /// Not abstained: the estimate follows audio evidence.
    #[default]
    None,
    Unvoiced,
    InsufficientEvidence,
    InsufficientFormantPeaks,
    ModelMismatch,
}

impl AbstainReason {
    pub fn as_str(self) -> &'static str {
        match self {
            AbstainReason::None => "audio_evidence",
            AbstainReason::Unvoiced => "unvoiced",
            AbstainReason::InsufficientEvidence => "insufficient_evidence",
            AbstainReason::InsufficientFormantPeaks => "insufficient_formant_peaks",
            AbstainReason::ModelMismatch => "model_mismatch",
        }
    }
}

/// One filtered estimate (`TemporalAtlasEstimate`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Estimate {
    pub coefficients: [f32; MAX_MODES],
    pub confidence: f32,
    pub abstained: bool,
    pub reason: AbstainReason,
}

/// `TemporalAtlasFilter`: on a voiced frame whose evidence clears the
/// model's abstention threshold the coefficients follow the raw estimate
/// at a rate that grows with the evidence and the confidence follows the
/// evidence; otherwise both decay toward the atlas mean and the frame is
/// abstained.
#[derive(Clone, Debug)]
pub struct TemporalAtlasFilter {
    coefficients: [f32; MAX_MODES],
    n_modes: usize,
    confidence: f32,
    limits: [[f32; 2]; MAX_MODES],
    threshold: f32,
    cfg: PosteriorConfig,
}

impl TemporalAtlasFilter {
    pub fn new(model: &ReducedModel, cfg: PosteriorConfig) -> Self {
        Self {
            coefficients: [0.0; MAX_MODES],
            n_modes: model.n_modes(),
            confidence: 0.0,
            limits: model.limits(),
            threshold: model.uncertainty.abstention_confidence_threshold,
            cfg,
        }
    }

    pub fn coefficients(&self) -> &[f32] {
        &self.coefficients[..self.n_modes]
    }

    pub fn update(&mut self, raw: &[f32; MAX_MODES], evidence: f32, voiced: bool) -> Estimate {
        let c = &self.cfg;
        if voiced && evidence >= self.threshold {
            let alpha = (c.filter_alpha_gain * evidence + c.filter_alpha_offset)
                .clamp(c.filter_alpha_min, c.filter_alpha_max);
            for ((cur, &target), &[lo, hi]) in self
                .coefficients
                .iter_mut()
                .zip(raw)
                .zip(&self.limits)
                .take(self.n_modes)
            {
                *cur = (*cur + (target - *cur) * alpha).clamp(lo, hi);
            }
            self.confidence += (evidence - self.confidence) * c.confidence_follow;
            Estimate {
                coefficients: self.coefficients,
                confidence: self.confidence,
                abstained: false,
                reason: AbstainReason::None,
            }
        } else {
            for v in &mut self.coefficients[..self.n_modes] {
                *v *= c.decay_coefficients;
            }
            self.confidence *= c.decay_confidence;
            Estimate {
                coefficients: self.coefficients,
                confidence: self.confidence,
                abstained: true,
                reason: if voiced {
                    AbstainReason::InsufficientEvidence
                } else {
                    AbstainReason::Unvoiced
                },
            }
        }
    }
}

static SHARED: OnceLock<Result<ReducedModel, String>> = OnceLock::new();

/// The compiled-in model, parsed once; its digest is checked first so a
/// changed file never loads under the recorded provenance.
pub fn shared() -> Result<&'static ReducedModel, String> {
    SHARED
        .get_or_init(|| {
            let digest = sha256_hex(JSON.as_bytes());
            if digest != JSON_SHA256 {
                return Err(format!(
                    "reduced_model.json digest {digest} is not the recorded {JSON_SHA256}"
                ));
            }
            ReducedModel::parse(JSON)
        })
        .as_ref()
        .map_err(Clone::clone)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> &'static ReducedModel {
        shared().unwrap()
    }

    #[test]
    fn the_shipped_model_is_schema_2_with_four_modes_over_32_sections() {
        let m = model();
        assert_eq!(m.schema_version, 2);
        assert_eq!(m.model_id, "vt3d-frozen-mri-pca-v0.7.0");
        assert_eq!((m.n_modes(), m.n_sections()), (4, 32));
        assert_eq!(
            (m.sample_rate_hz, m.frame_size, m.hop_size),
            (16_000, 1024, 512)
        );
        assert_eq!(m.reference_formants_hz, [716.0, 1922.0, 3074.0, 4374.0]);
        assert_eq!(m.uncertainty.abstention_confidence_threshold, 0.22);
        assert_eq!(m.uncertainty.maximum_relative_area_std, 0.65);
        assert!(!m.atlas.scientific_release_ready);
        assert_eq!(m.atlas.independent_expert_acceptances, 0);
        assert_eq!(m.atlas.subjects, 5);
        assert_eq!(m.acoustic_inverse.ridge, 6.2);
        assert_eq!(m.limits(), [[-2.0, 2.0]; 4]);
    }

    /// Contract (`inferArea`, schema 2): at the reference formants every
    /// coefficient is 0; +100 Hz on F1..F3 gives the row sums of the first
    /// three columns × 100, clamped to ±2. F4 is not measured by VoxLabs
    /// (three formants), so its column never enters.
    #[test]
    fn inference_is_the_clamped_linear_map_from_the_json() {
        let m = model();
        let mut c = [9.0; MAX_MODES];
        m.infer_coefficients(&[716.0, 1922.0, 3074.0], &mut c);
        assert_eq!(c, [0.0; 4]);
        m.infer_coefficients(&[816.0, 2022.0, 3174.0], &mut c);
        for i in 0..4 {
            let want: f32 = (0..3).map(|j| m.formant_to_mode[i][j] * 100.0).sum();
            assert!(
                (c[i] - want.clamp(-2.0, 2.0)).abs() < 1e-6,
                "mode {i}: {c:?}"
            );
        }
        // Far off the reference: clamped at the limits.
        m.infer_coefficients(&[5000.0, 200.0, 8000.0], &mut c);
        assert!(c.iter().all(|v| (-2.0..=2.0).contains(v)), "{c:?}");
        assert!(c.iter().any(|v| v.abs() == 2.0), "{c:?}");
    }

    /// Contract (`areaFromCoefficients`): zero coefficients give the mean;
    /// one SD of mode 1 multiplies section k by exp(mode_1[k]); the clamp
    /// holds at [0.2, 5] × mean.
    #[test]
    fn area_synthesis_is_the_exponentiated_log_mean_plus_modes() {
        let m = model();
        let cfg = PosteriorConfig::DEFAULT;
        let mut a = [0.0f32; 32];
        m.area_from_coefficients(&[0.0; 4], &cfg, &mut a).unwrap();
        for (got, mean) in a.iter().zip(&m.area_mean_cm2) {
            assert!((got - mean).abs() < 1e-6 * mean);
        }
        m.area_from_coefficients(&[1.0, 0.0, 0.0, 0.0], &cfg, &mut a)
            .unwrap();
        for (k, got) in a.iter().enumerate() {
            let want = (m.area_mean_cm2[k] * m.area_modes[0][k].exp())
                .clamp(0.2 * m.area_mean_cm2[k], 5.0 * m.area_mean_cm2[k]);
            assert!((got - want).abs() < 1e-5 * want, "section {k}");
        }
        // Beyond the limits the coefficient is clamped before it enters.
        let mut b = [0.0f32; 32];
        m.area_from_coefficients(&[7.0, 0.0, 0.0, 0.0], &cfg, &mut b)
            .unwrap();
        let mut two = [0.0f32; 32];
        m.area_from_coefficients(&[2.0, 0.0, 0.0, 0.0], &cfg, &mut two)
            .unwrap();
        assert_eq!(b, two);
        assert!(
            m.area_from_coefficients(&[0.0; 4], &cfg, &mut [0.0; 8])
                .is_err()
        );
    }

    /// Contract (`TemporalAtlasFilter.update`), hand-stepped: an evidence
    /// frame at 0.5 follows at α = 0.31 and lifts the confidence by 0.35 of
    /// the way; an unvoiced frame decays by 0.9 / 0.82.
    #[test]
    fn the_filter_follows_evidence_and_decays_on_abstention() {
        let m = model();
        let mut f = TemporalAtlasFilter::new(m, PosteriorConfig::DEFAULT);
        let raw = [1.0, -1.0, 0.5, 2.5];
        let e = f.update(&raw, 0.5, true);
        assert!(!e.abstained);
        assert_eq!(e.reason, AbstainReason::None);
        let alpha = 0.38 * 0.5 + 0.12;
        for (c, r) in e.coefficients.iter().zip(&raw) {
            let want = (r * alpha).clamp(-2.0, 2.0);
            assert!((c - want).abs() < 1e-6, "{e:?}");
        }
        assert!((e.confidence - 0.5 * 0.35).abs() < 1e-6);
        let u = f.update(&raw, 0.9, false);
        assert!(u.abstained);
        assert_eq!(u.reason, AbstainReason::Unvoiced);
        for (after, before) in u.coefficients.iter().zip(&e.coefficients) {
            assert!((after - before * 0.9).abs() < 1e-6);
        }
        assert!((u.confidence - e.confidence * 0.82).abs() < 1e-6);
        // Voiced but below the model's threshold: insufficient evidence.
        let w = f.update(&raw, 0.2, true);
        assert_eq!(
            (w.abstained, w.reason),
            (true, AbstainReason::InsufficientEvidence)
        );
        // Evidence beyond 0.895 saturates the follow rate at 0.46.
        let s = f.update(&raw, 0.95, true);
        let alpha_max = 0.46;
        let want0 = (w.coefficients[0] + (raw[0] - w.coefficients[0]) * alpha_max).clamp(-2.0, 2.0);
        assert!((s.coefficients[0] - want0).abs() < 1e-6);
    }

    #[test]
    fn a_tampered_model_is_refused() {
        let bad = JSON.replace("\"schema_version\": 2", "\"schema_version\": 3");
        assert!(bad != JSON);
        assert!(ReducedModel::parse(&bad).is_err());
        let long_hop = JSON.replace("\"hop_size\": 512", "\"hop_size\": 4096");
        assert!(long_hop != JSON);
        assert!(ReducedModel::parse(&long_hop).is_err(), "hop above frame");
    }
}
