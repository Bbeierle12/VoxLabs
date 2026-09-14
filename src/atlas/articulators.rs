//! The articulator readout: Vocal Tract Lab's `ArticulatorPosterior.infer`
//! — the mean log ratio of the estimated area to the atlas mean over six
//! regions of the normalized tract, each clamped to ±1, combined into jaw,
//! lip and tongue readings. A consumer of the area function, not a stage
//! (completion-plan call 4). Interpretation, in the app's words: an
//! audio-conditioned estimate; anatomy is not observed.

use crate::config::ArticulatorConfig;

#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ArticulatorPosterior {
    pub jaw_opening: f32,
    pub lip_aperture: f32,
    pub tongue_front_back: f32,
    pub tongue_dorsum: f32,
    pub tongue_root: f32,
    pub pharynx_width: f32,
    pub epilarynx_width: f32,
}

/// Mean log(area / mean) over the sections in `[lo, hi)` of the tract
/// (fractions of its length), clamped to ±1.
fn regional(area: &[f32], mean: &[f32], lo: f32, hi: f32, floor: f32) -> f32 {
    let n = area.len();
    let start = ((lo * n as f32) as usize).min(n - 1);
    let end = ((hi * n as f32) as usize).clamp(start + 1, n);
    let mut sum = 0.0f32;
    for i in start..end {
        sum += (area[i] / mean[i]).max(floor).ln();
    }
    (sum / (end - start) as f32).clamp(-1.0, 1.0)
}

/// `area` and `mean` are the section areas (≥ 16 sections, equal length).
pub fn infer(
    area: &[f32],
    mean: &[f32],
    cfg: &ArticulatorConfig,
) -> Result<ArticulatorPosterior, String> {
    if area.len() != mean.len() || area.len() < 16 {
        return Err(format!(
            "articulators: need ≥ 16 matched sections, got {} and {}",
            area.len(),
            mean.len()
        ));
    }
    let r = |lo, hi| regional(area, mean, lo, hi, cfg.ratio_floor);
    let epilarynx = r(cfg.epilarynx_lo, cfg.epilarynx_hi);
    let pharynx = r(cfg.pharynx_lo, cfg.pharynx_hi);
    let root = r(cfg.tongue_root_lo, cfg.tongue_root_hi);
    let dorsum = r(cfg.tongue_dorsum_lo, cfg.tongue_dorsum_hi);
    let front = r(cfg.tongue_front_lo, cfg.tongue_front_hi);
    let lips = r(cfg.lips_lo, cfg.lips_hi);
    Ok(ArticulatorPosterior {
        jaw_opening: (cfg.jaw_lip_weight * lips + cfg.jaw_front_weight * front).clamp(-1.0, 1.0),
        lip_aperture: lips,
        tongue_front_back: (front - root).clamp(-1.0, 1.0),
        tongue_dorsum: (-dorsum).clamp(-1.0, 1.0),
        tongue_root: (-root).clamp(-1.0, 1.0),
        pharynx_width: pharynx,
        epilarynx_width: epilarynx,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mean_shape_reads_zero_everywhere_and_a_wider_front_moves_the_jaw() {
        let cfg = ArticulatorConfig::DEFAULT;
        let mean: Vec<f32> = (0..32).map(|i| 1.0 + i as f32 * 0.05).collect();
        let z = infer(&mean, &mean, &cfg).unwrap();
        assert_eq!(z, ArticulatorPosterior::default());
        // Double the last 4 sections (the lips, 0.88..1.0 of 32 = sections 28..32).
        let mut open = mean.clone();
        for a in &mut open[28..] {
            *a *= 2.0;
        }
        let o = infer(&open, &mean, &cfg).unwrap();
        let ln2 = 2f32.ln();
        assert!((o.lip_aperture - ln2).abs() < 1e-6, "{o:?}");
        // Front region 0.68..0.88 → sections 21..28: untouched.
        assert!((o.jaw_opening - cfg.jaw_lip_weight * ln2).abs() < 1e-6);
        assert_eq!(o.tongue_root, 0.0);
        assert!(infer(&mean[..8], &mean[..8], &cfg).is_err());
    }
}
