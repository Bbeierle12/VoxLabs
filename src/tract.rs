//! Runtime side of the Story two-mode tract model: area synthesis, a
//! lossless chain-matrix resonance solver, the (q1, q2) inversion grid, and
//! the uniform-tube VTL estimator. Basis data and the honesty boundary live
//! in [`crate::tract_data`]; read that header before changing any claim this
//! module's consumers make.

pub use crate::tract_data::{ADULT_FEMALE, ADULT_MALE, N_SECTIONS, TractBasis, VOWEL_ANCHORS};

use crate::config::TractConfig;
use crate::config::consts::{HALF, QUARTER_WAVE_DENOM, QUARTER_WAVE_ODD_MULTIPLES, TWO};
use std::f32::consts::TAU;

/// Stage configuration (see `config::TractConfig` and `pipeline.toml`).
const TRACT: TractConfig = TractConfig::DEFAULT;

/// Speed of sound, cm/s at vocal-tract temperature (Story's own value).
pub(crate) const SPEED_OF_SOUND_CM_S: f32 = TRACT.speed_of_sound_cm_s;
/// Resonance search band and bracketing step, Hz; bisection refinements.
const RES_SWEEP_LO_HZ: f32 = TRACT.sweep_lo_hz;
const RES_SWEEP_HI_HZ: f32 = TRACT.sweep_hi_hz;
const RES_SWEEP_STEP_HZ: f32 = TRACT.sweep_step_hz;
const RES_BISECT_ITERS: usize = TRACT.bisect_iters;
/// Resonances the solver returns (fR1..fR3).
pub const N_RESONANCES: usize = TRACT.n_resonances;
/// Diameter clamp (cm) before squaring — see `TractConfig::min_diameter_cm`.
const MIN_DIAMETER_CM: f32 = TRACT.min_diameter_cm;

/// Published (q1, q2) span (Story 2018, Table II: q1 in [-5.10, 3.86], q2 in
/// [-2.69, 2.22]) plus margin, so the grid covers the whole vowel space
/// without extrapolating far beyond what the PCA saw.
pub const Q1_MIN: f32 = TRACT.q1_min;
pub const Q1_MAX: f32 = TRACT.q1_max;
pub const Q2_MIN: f32 = TRACT.q2_min;
pub const Q2_MAX: f32 = TRACT.q2_max;

/// Inversion-grid resolution per axis at runtime (41 x 41 = 1681 forward
/// solves, built once on a background thread).
pub const GRID_N: usize = TRACT.grid_n;

/// Reject an inversion whose nearest grid node is farther than this in the
/// normalized formant metric of [`TractGrid::invert`] — the measured pair
/// lies outside the model's vowel space (or is a bad estimate), and snapping
/// it to the nearest edge would draw a confident wrong shape. Display
/// convention, sized at a few grid cells.
const INVERT_MAX_DIST: f32 = TRACT.invert_max_dist;

/// Formant-distance normalization (Hz) for the inversion metric: roughly the
/// in-band spacing of grid nodes along each axis, so one "unit" of distance
/// is comparable in F1 and F2.
const INVERT_F1_SCALE_HZ: f32 = TRACT.invert_f1_scale_hz;
const INVERT_F2_SCALE_HZ: f32 = TRACT.invert_f2_scale_hz;

/// VTL sanity bounds, cm. Outside published human range (infant ~8, tall
/// adult male ~19-20; Story 2018 cohort spans 8.45-18.8) the estimate is a
/// measurement artifact, not anatomy.
const VTL_MIN_CM: f32 = TRACT.vtl_min_cm;
const VTL_MAX_CM: f32 = TRACT.vtl_max_cm;

/// Diameter function D(i) = Omega + q1*phi1 + q2*phi2, cm, clamped positive.
/// This is what the pseudo-midsagittal display renders directly (Story
/// 2007b renders exactly this profile).
pub fn diameters(basis: &TractBasis, q1: f32, q2: f32) -> [f32; N_SECTIONS] {
    std::array::from_fn(|i| {
        (basis.omega[i] + q1 * basis.phi1[i] + q2 * basis.phi2[i]).max(MIN_DIAMETER_CM)
    })
}

/// Area function V(i) = (pi/4) * D(i)^2, cm^2 — Story 2018 Eq. (1).
pub fn area_function(basis: &TractBasis, q1: f32, q2: f32) -> [f32; N_SECTIONS] {
    let d = diameters(basis, q1, q2);
    std::array::from_fn(|i| std::f32::consts::FRAC_PI_4 * d[i] * d[i])
}

/// First three acoustic resonances (Hz) of an area function of total length
/// `vtl_cm`, under the standard lossless idealization: plane waves, rigid
/// walls, flow source at a closed glottis, ideal pressure release at the
/// lips. Resonances are the zeros of the chain matrix's D term.
///
/// Lossless on purpose: resonance *frequencies* are what the inversion
/// needs, and the loss models that fix bandwidths move the frequencies by
/// far less than the measurement error of the formants being inverted.
/// Returns `None` if fewer than three zeros lie in the sweep band.
pub fn resonances(areas: &[f32; N_SECTIONS], vtl_cm: f32) -> Option<[f32; N_RESONANCES]> {
    let mut roots = [0.0f32; N_RESONANCES];
    let found = resonances_up_to(areas, vtl_cm, RES_SWEEP_HI_HZ, &mut roots);
    (found == N_RESONANCES).then_some(roots)
}

/// The same sweep with a caller-supplied ceiling and slot count: fills
/// `out` with the resonances found in `[sweep_lo, hi_hz]` in ascending
/// order and returns how many. `resonances` is this with the configured
/// band and three slots; Experiment 3 asks for four up to
/// `validation.forward_sweep_hi_hz` because the atlas map is built on F4.
pub fn resonances_up_to(
    areas: &[f32; N_SECTIONS],
    vtl_cm: f32,
    hi_hz: f32,
    out: &mut [f32],
) -> usize {
    if !(vtl_cm.is_finite() && vtl_cm > 0.0) || out.is_empty() {
        return 0;
    }
    let seg_len = vtl_cm / N_SECTIONS as f32;

    // Chain-matrix D term at frequency f. For a lossless line each section
    // matrix has the structure [[a, jb], [jc, d]] with a, b, c, d real; that
    // structure is closed under multiplication, so D stays real and roots
    // can be bracketed by sign change. Characteristic impedance enters only
    // as 1/A ratios — the common rho*c factor cancels in D's zeros.
    let d_term = |f: f32| -> f32 {
        let theta = TAU * f * seg_len / SPEED_OF_SOUND_CM_S;
        let (sin_t, cos_t) = theta.sin_cos();
        // Running product T, glottis -> lips: T = [[a, jb], [jc, d]].
        let (mut a, mut b, mut c, mut d) = (1.0f32, 0.0f32, 0.0f32, 1.0f32);
        for &area in areas.iter() {
            let z = 1.0 / area; // per-section impedance, common factor dropped
            let (sa, sb, sc, sd) = (cos_t, z * sin_t, sin_t / z, cos_t);
            let na = a * sa - b * sc;
            let nb = a * sb + b * sd;
            let nc = c * sa + d * sc;
            let nd = d * sd - c * sb;
            (a, b, c, d) = (na, nb, nc, nd);
        }
        d
    };

    let slots = out.len();
    let mut found = 0;
    let mut f_prev = RES_SWEEP_LO_HZ;
    let mut d_prev = d_term(f_prev);
    let mut f = f_prev + RES_SWEEP_STEP_HZ;
    while f <= hi_hz && found < slots {
        let d_now = d_term(f);
        if d_prev == 0.0 || d_prev.signum() != d_now.signum() {
            // Bracketed: bisect.
            let (mut lo, mut hi) = (f_prev, f);
            let mut d_lo = d_prev;
            for _ in 0..RES_BISECT_ITERS {
                let mid = HALF * (lo + hi);
                let d_mid = d_term(mid);
                if d_lo.signum() != d_mid.signum() {
                    hi = mid;
                } else {
                    lo = mid;
                    d_lo = d_mid;
                }
            }
            out[found] = HALF * (lo + hi);
            found += 1;
        }
        f_prev = f;
        d_prev = d_now;
        f += RES_SWEEP_STEP_HZ;
    }
    found
}

/// Precomputed forward map over the (q1, q2) plane: resonances (fR1, fR2) at
/// each node, inverted at runtime by nearest-node search plus
/// inverse-distance weighting over the 3x3 neighborhood. Story 2007b's
/// near one-to-one property is what makes this lookup meaningful.
pub struct TractGrid {
    n1: usize,
    n2: usize,
    f1: Vec<f32>,
    f2: Vec<f32>,
}

impl TractGrid {
    /// Node coordinate along an axis.
    fn q_at(min: f32, max: f32, n: usize, idx: usize) -> f32 {
        min + (max - min) * idx as f32 / (n - 1) as f32
    }

    /// Builds the forward grid: `n1 x n2` solves of the lossless model. At
    /// the runtime resolution this is ~1700 solves — sub-second in release,
    /// done once off the UI thread.
    pub fn build(basis: &TractBasis, n1: usize, n2: usize) -> Self {
        assert!(n1 >= TRACT.grid_min_n && n2 >= TRACT.grid_min_n);
        let mut f1 = vec![0.0f32; n1 * n2];
        let mut f2 = vec![0.0f32; n1 * n2];
        for i in 0..n1 {
            let q1 = Self::q_at(Q1_MIN, Q1_MAX, n1, i);
            for j in 0..n2 {
                let q2 = Self::q_at(Q2_MIN, Q2_MAX, n2, j);
                let areas = area_function(basis, q1, q2);
                if let Some([r1, r2, _]) = resonances(&areas, basis.vtl_cm) {
                    f1[i * n2 + j] = r1;
                    f2[i * n2 + j] = r2;
                }
                // A failed solve leaves 0.0 — infinitely far in the metric,
                // so inversion can never select it.
            }
        }
        Self { n1, n2, f1, f2 }
    }

    /// Inverts a measured (F1, F2) pair (already scaled into this basis's
    /// length reference) to mode coefficients. `None` when the pair lies
    /// outside the model's vowel space — the honest answer, not a snap to
    /// the nearest edge.
    pub fn invert(&self, f1_hz: f32, f2_hz: f32) -> Option<(f32, f32)> {
        if !(f1_hz.is_finite() && f2_hz.is_finite() && f1_hz > 0.0 && f2_hz > 0.0) {
            return None;
        }
        let dist = |idx: usize| -> f32 {
            let (g1, g2) = (self.f1[idx], self.f2[idx]);
            if g1 <= 0.0 {
                return f32::INFINITY;
            }
            let d1 = (f1_hz - g1) / INVERT_F1_SCALE_HZ;
            let d2 = (f2_hz - g2) / INVERT_F2_SCALE_HZ;
            d1 * d1 + d2 * d2
        };

        let (mut best, mut best_d) = (0usize, f32::INFINITY);
        for idx in 0..self.f1.len() {
            let d = dist(idx);
            if d < best_d {
                best_d = d;
                best = idx;
            }
        }
        if best_d > INVERT_MAX_DIST {
            return None;
        }

        // Inverse-distance weighting over the 3x3 neighborhood of the best
        // node: sub-cell resolution without trusting any single solve.
        let (bi, bj) = (best / self.n2, best % self.n2);
        let (mut wq1, mut wq2, mut wsum) = (0.0f32, 0.0f32, 0.0f32);
        for di in -1isize..=1 {
            for dj in -1isize..=1 {
                let i = bi as isize + di;
                let j = bj as isize + dj;
                if i < 0 || j < 0 || i >= self.n1 as isize || j >= self.n2 as isize {
                    continue;
                }
                let idx = i as usize * self.n2 + j as usize;
                let d = dist(idx);
                if !d.is_finite() {
                    continue;
                }
                let w = 1.0 / (d + TRACT.idw_eps);
                wq1 += w * Self::q_at(Q1_MIN, Q1_MAX, self.n1, i as usize);
                wq2 += w * Self::q_at(Q2_MIN, Q2_MAX, self.n2, j as usize);
                wsum += w;
            }
        }
        (wsum > 0.0).then(|| (wq1 / wsum, wq2 / wsum))
    }
}

/// Vocal tract length estimate from one frame's F2 and F3, via the uniform
/// closed–open tube relations L = (2n-1)·c / (4·Fn). F1 is excluded and F3
/// outweighs F2 because higher formants vary less with articulation and so
/// carry more anatomy per hertz (the principle behind Lammert & Narayanan
/// 2015's estimator, whose real-speech accuracy — ±1.3 cm RMS — is also the
/// right expectation to attach to this number). Weights are an engineering
/// choice under that principle, not a published constant.
///
/// Feed this identity-grade frames only (see `math::formant_grade`): a
/// harmonic-attracted F3 poisons the estimate exactly like any other use.
pub fn vtl_from_formants(f2_hz: f32, f3_hz: f32) -> Option<f32> {
    if !(f2_hz.is_finite() && f3_hz.is_finite() && f2_hz > 0.0 && f3_hz > 0.0) {
        return None;
    }
    const W_F2: f32 = TRACT.vtl_weight_f2;
    const W_F3: f32 = TRACT.vtl_weight_f3;
    // F2 is the second closed–open mode (3c/4L), F3 the third (5c/4L).
    let [_, odd_f2, odd_f3] = QUARTER_WAVE_ODD_MULTIPLES;
    let l2 = odd_f2 * SPEED_OF_SOUND_CM_S / (QUARTER_WAVE_DENOM * f2_hz);
    let l3 = odd_f3 * SPEED_OF_SOUND_CM_S / (QUARTER_WAVE_DENOM * f3_hz);
    let l = W_F2 * l2 + W_F3 * l3;
    (VTL_MIN_CM..=VTL_MAX_CM).contains(&l).then_some(l)
}

/// Picks the published basis nearest an estimated tract length: female
/// (15.53 cm) below the midpoint of the two adult bases, male (17.6 cm)
/// above. With no estimate, the male basis — the only one derived directly
/// from MRI rather than by length-warping.
pub fn basis_for_vtl(vtl_cm: Option<f32>) -> &'static TractBasis {
    match vtl_cm {
        Some(l) if l < (ADULT_FEMALE.vtl_cm + ADULT_MALE.vtl_cm) / TWO => &ADULT_FEMALE,
        _ => &ADULT_MALE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Story 2018 Table II — the ten vowels the model was built from.
    const TABLE_II: [(&str, f32, f32); 10] = [
        ("i", -5.10, 0.88),
        ("ɪ", -2.57, 0.93),
        ("ɛ", -1.07, 1.22),
        ("æ", 0.66, 2.22),
        ("ʌ", 2.55, 0.10),
        ("ɑ", 3.86, 1.35),
        ("ɔ", 3.47, -0.45),
        ("o", 0.00, -2.69),
        ("ʊ", 1.68, -1.87),
        ("u", -3.48, -1.70),
    ];

    /// A uniform closed–open tube resonates at (2n-1)·c/4L; the multi-
    /// section chain matrix must reproduce that, which validates both the
    /// section composition and the root search.
    #[test]
    fn uniform_tube_resonates_at_odd_quarter_wave_multiples() {
        let areas = [3.0f32; N_SECTIONS];
        let l = 17.5;
        let [r1, r2, r3] = resonances(&areas, l).expect("three resonances");
        let f1 = SPEED_OF_SOUND_CM_S / (4.0 * l);
        for (got, want) in [(r1, f1), (r2, 3.0 * f1), (r3, 5.0 * f1)] {
            assert!(
                (got - want).abs() < 0.01 * want,
                "expected {want:.0} Hz, got {got:.1}"
            );
        }
    }

    /// Every published vowel must synthesize to a physically meaningful
    /// area function in both bases — positive everywhere, no clamped
    /// negative-diameter artifacts at the tract's working extremes.
    #[test]
    fn published_vowels_have_positive_areas_in_both_bases() {
        for basis in [&ADULT_MALE, &ADULT_FEMALE] {
            for (name, q1, q2) in TABLE_II {
                let areas = area_function(basis, q1, q2);
                assert!(
                    areas.iter().all(|&a| a.is_finite() && a > 0.0),
                    "vowel {name} produced a non-positive area"
                );
            }
        }
    }

    /// Gross acoustic sanity against a century of phonetics: /i/ is a low-F1
    /// high-F2 vowel, /ɑ/ the reverse. If the basis transcription had rows
    /// swapped or a sign flipped, this is what breaks.
    #[test]
    fn corner_vowels_land_in_the_right_formant_quadrants() {
        let i_areas = area_function(&ADULT_MALE, -5.10, 0.88);
        let [i_f1, i_f2, _] = resonances(&i_areas, ADULT_MALE.vtl_cm).unwrap();
        let a_areas = area_function(&ADULT_MALE, 3.86, 1.35);
        let [a_f1, a_f2, _] = resonances(&a_areas, ADULT_MALE.vtl_cm).unwrap();

        assert!(i_f1 < 400.0, "/i/ F1 = {i_f1:.0}");
        assert!(i_f2 > 1800.0, "/i/ F2 = {i_f2:.0}");
        assert!(a_f1 > 600.0, "/ɑ/ F1 = {a_f1:.0}");
        assert!(a_f2 < 1400.0, "/ɑ/ F2 = {a_f2:.0}");
        assert!(i_f1 < a_f1 && i_f2 > a_f2);
    }

    /// Forward-then-invert must recover the mode coefficients of the
    /// published vowels to within the coarse test grid's resolution — the
    /// working form of Story 2007b's one-to-one property.
    #[test]
    fn inversion_round_trips_the_published_vowels() {
        // Coarser than runtime (21x21 vs 41x41) to keep the test fast;
        // tolerance scaled accordingly.
        let grid = TractGrid::build(&ADULT_MALE, 21, 21);
        for (name, q1, q2) in [("i", -5.10, 0.88), ("ɑ", 3.86, 1.35), ("u", -3.48, -1.70)] {
            let areas = area_function(&ADULT_MALE, q1, q2);
            let [f1, f2, _] = resonances(&areas, ADULT_MALE.vtl_cm).unwrap();
            let (r1, r2) = grid
                .invert(f1, f2)
                .unwrap_or_else(|| panic!("vowel {name} failed to invert"));
            assert!(
                (r1 - q1).abs() < 0.6 && (r2 - q2).abs() < 0.5,
                "vowel {name}: ({q1}, {q2}) round-tripped to ({r1:.2}, {r2:.2})"
            );
        }
    }

    /// A formant pair far outside the vowel space must refuse to invert
    /// rather than snap to the nearest edge and draw a confident shape.
    #[test]
    fn out_of_space_formants_refuse_to_invert() {
        let grid = TractGrid::build(&ADULT_MALE, 21, 21);
        assert!(grid.invert(3000.0, 600.0).is_none()); // F1 > F2 nonsense
        assert!(grid.invert(f32::NAN, 1500.0).is_none());
        assert!(grid.invert(-100.0, 1500.0).is_none());
    }

    /// The VTL estimator must recover a uniform tube's length from its own
    /// resonances — the case where its model assumptions hold exactly.
    #[test]
    fn vtl_estimator_recovers_a_uniform_tube() {
        for l in [14.0f32, 16.0, 17.5] {
            let areas = [3.0f32; N_SECTIONS];
            let [_, f2, f3] = resonances(&areas, l).unwrap();
            let est = vtl_from_formants(f2, f3).expect("in range");
            assert!((est - l).abs() < 0.3, "true {l}, estimated {est:.2}");
        }
    }

    #[test]
    fn vtl_estimator_rejects_garbage() {
        assert!(vtl_from_formants(0.0, 2500.0).is_none());
        assert!(vtl_from_formants(f32::NAN, 2500.0).is_none());
        // Formants of a 2 cm "tract" — outside human range.
        assert!(vtl_from_formants(13_000.0, 22_000.0).is_none());
    }

    /// Basis selection: below the midpoint uses the female basis, above (or
    /// unknown) the male.
    #[test]
    fn basis_selection_by_vtl() {
        assert!((basis_for_vtl(Some(15.0)).vtl_cm - 15.53).abs() < 0.01);
        assert!((basis_for_vtl(Some(18.0)).vtl_cm - 17.6).abs() < 0.01);
        assert!((basis_for_vtl(None).vtl_cm - 17.6).abs() < 0.01);
    }
}
