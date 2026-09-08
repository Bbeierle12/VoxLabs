//! Story two-mode vocal-tract model configuration (`tract.rs`,
//! `tract_data.rs` scalars) and the grid-inverse stage. The published basis
//! tables stay in `tract_data.rs`; only the solver and inversion parameters
//! live here.

use super::stage_config;

stage_config! {
    /// Area synthesis, lossless chain-matrix resonance solver, (q1, q2)
    /// inversion grid, and the uniform-tube VTL estimator.
    pub struct TractConfig, section = "tract" {
        /// Speed of sound, cm/s at vocal-tract temperature — the value Story's
        /// own resonance calculations use. Range: 33_000.0..=36_000.0.
        speed_of_sound_cm_s: f32 = 35_000.0,
        /// Resonance search band, low edge, Hz (below any adult fR1).
        /// Range: 50.0..=200.0.
        sweep_lo_hz: f32 = 100.0,
        /// Resonance search band, high edge, Hz (covers fR3, keeps the
        /// plane-wave assumption honest). Range: 3000.0..=5000.0.
        sweep_hi_hz: f32 = 4_000.0,
        /// Sweep step for bracketing sign changes of the chain-matrix D term,
        /// Hz; far below the closest in-band resonance spacing. Range: 5.0..=50.0.
        sweep_step_hz: f32 = 20.0,
        /// Bisection refinements per bracketed root (interval shrinks 2^−n).
        /// Range: 10..=40.
        bisect_iters: usize = 30,
        /// Resonances the solver returns (fR1..fR3). Range: exactly 3.
        n_resonances: usize = 3,
        /// Diameter clamp, cm, before squaring — a negative mode sum would
        /// square into a spurious open passage. Range: 0.01..=0.2.
        min_diameter_cm: f32 = 0.05,
        /// (q1, q2) inversion-grid span: Story 2018 Table II's published range
        /// plus margin. q1 low edge. Range: -7.0..=-5.1.
        q1_min: f32 = -5.6,
        /// q1 high edge. Range: 3.86..=6.0.
        q1_max: f32 = 4.4,
        /// q2 low edge. Range: -4.0..=-2.69.
        q2_min: f32 = -3.1,
        /// q2 high edge. Range: 2.22..=4.0.
        q2_max: f32 = 2.7,
        /// Inversion-grid resolution per axis at runtime (41 × 41 = 1681
        /// forward solves, once on a background thread). Range: grid_min_n..=81.
        grid_n: usize = 41,
        /// Fewest nodes per axis a grid may be built with (the node spacing
        /// divides by n − 1). Range: exactly 2.
        grid_min_n: usize = 2,
        /// Reject an inversion whose nearest node is farther than this in the
        /// normalized formant metric (a few grid cells). Range: 1.0..=10.0.
        invert_max_dist: f32 = 4.0,
        /// F1 distance normalization, Hz (≈ in-band node spacing).
        /// Range: 50.0..=300.0.
        invert_f1_scale_hz: f32 = 150.0,
        /// F2 distance normalization, Hz. Range: 100.0..=600.0.
        invert_f2_scale_hz: f32 = 300.0,
        /// Added to a node's distance in the inverse-distance weights so an
        /// exact hit does not divide by zero. Range: 1e-6..=1e-1.
        idw_eps: f32 = 1e-3,
        /// VTL sanity bounds, cm: below this is not adult anatomy.
        /// Range: 6.0..=10.0.
        vtl_min_cm: f32 = 8.0,
        /// VTL upper bound, cm. Range: 19.0..=25.0.
        vtl_max_cm: f32 = 22.0,
        /// Weight of the F2-derived length in the VTL estimate (F3 gets the
        /// rest; higher formants carry more anatomy per hertz). Range: 0.0..=0.5.
        vtl_weight_f2: f32 = 0.3,
        /// Weight of the F3-derived length. Range: 0.5..=1.0; with vtl_weight_f2 sums to 1.
        vtl_weight_f3: f32 = 0.7,
    }
}

stage_config! {
    /// The grid-inverse stage (`pipeline::stages::inverse`): measured
    /// (F1, F2) → Story mode coefficients through the precomputed grid.
    pub struct InverseConfig, section = "inverse" {
        /// EMA coefficient for the per-frame vocal-tract-length estimate that
        /// picks the basis and scales the formants into its length reference.
        /// Anatomy accumulates slowly; identity-grade frames only.
        /// Range: 0.01..=0.3.
        vtl_ema_alpha: f32 = 0.05,
    }
}
