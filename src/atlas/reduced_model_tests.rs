//! The reduced model's contract: the app's formulas, hand-stepped.

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
