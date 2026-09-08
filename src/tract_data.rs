//! Story two-mode vocal tract model: area-function synthesis, resonance
//! computation, and (q1, q2) inversion from measured formants.
//!
//! Basis data is transcribed verbatim from Story, Vorperian, Bunton &
//! Durtschi (2018), "An age-dependent vocal tract model for males and
//! females based on anatomic measurements", JASA 143(5):3079-3102
//! (open access, PMC5966313): Appendix Table III (adult male, the original
//! MRI-derived model) and Table IV (derived adult female), 44 elements each,
//! element 1 just above the glottis and element 44 the lip termination; and
//! Table II (per-vowel mode coefficients). The model is
//!
//! ```text
//! V(i) = (pi/4) * [ Omega(i) + q1*phi1(i) + q2*phi2(i) ]^2
//! ```
//!
//! -- their Eq. (1): PCA runs on *diameters*, and the square converts to
//! area. Story 2007b (JASA 121:3770) shows the (q1, q2) -> (fR1, fR2) map is
//! near one-to-one at fixed tract length, which is what makes inversion from
//! measured formants meaningful at all.
//!
//! HONESTY BOUNDARY (do not soften in UI copy): this is one adult male's
//! MRI-derived shape space (the female version is an anatomically
//! length-warped derivation of it, not independent data). Inverting a
//! singer's (F1, F2) through it yields "a model tract whose resonances match
//! yours" -- a low-dimensional shape hypothesis, never a picture of the
//! singer's anatomy. The literal geometry is not recoverable from audio:
//! A(x) and 1/A(L-x) are acoustically identical (Qin & Carreira-Perpinan
//! 2007), and formants determine only part of the shape (Mermelstein 1967;
//! Sondhi 1979).

//!
//! This module is DATA: the published tables, transcribed. The solver and
//! inversion parameters (speed of sound, sweep band, clamps, grid) are
//! stage configuration in `config::TractConfig` / `pipeline.toml`.

/// Number of area elements, glottis (1) to lips (44). Fixed by the dataset.
pub const N_SECTIONS: usize = 44;

/// Corner-vowel anchors for display, from Story 2018 Table II — the model's
/// own landmarks (one speaker's vowels), drawn as context in the log map,
/// never as targets for this singer.
pub const VOWEL_ANCHORS: [(&str, f32, f32); 5] = [
    ("i", -5.10, 0.88),
    ("æ", 0.66, 2.22),
    ("ɑ", 3.86, 1.35),
    ("o", 0.00, -2.69),
    ("u", -3.48, -1.70),
];

/// One anatomical basis: neutral diameter function, two modes, and the
/// tract length the elements imply.
pub struct TractBasis {
    pub omega: [f32; N_SECTIONS],
    pub phi1: [f32; N_SECTIONS],
    pub phi2: [f32; N_SECTIONS],
    /// Total tract length, cm (element length x 44).
    pub vtl_cm: f32,
}

/// Story 2018, Table III, adult-male columns (the original 1996-derived model). L(i) = 0.400 cm, VTL 17.6 cm.
pub const ADULT_MALE: TractBasis = TractBasis {
    omega: [
        0.8, 0.74, 0.71, 0.72, 0.85, 1.1, 1.35, 1.47, 1.44, 1.39, 1.38, 1.42, 1.46, 1.49, 1.49,
        1.49, 1.5, 1.53, 1.57, 1.59, 1.58, 1.57, 1.56, 1.56, 1.53, 1.5, 1.46, 1.46, 1.52, 1.6,
        1.68, 1.76, 1.86, 1.97, 2.06, 2.13, 2.17, 2.13, 2.01, 1.84, 1.69, 1.55, 1.43, 1.31,
    ],
    phi1: [
        0.004, -0.014, -0.029, -0.042, -0.054, -0.066, -0.078, -0.091, -0.105, -0.12, -0.137,
        -0.153, -0.17, -0.184, -0.195, -0.203, -0.205, -0.201, -0.191, -0.174, -0.149, -0.118,
        -0.082, -0.04, 0.005, 0.051, 0.097, 0.141, 0.18, 0.213, 0.238, 0.255, 0.261, 0.257, 0.244,
        0.221, 0.192, 0.158, 0.122, 0.089, 0.061, 0.043, 0.038, 0.047,
    ],
    phi2: [
        0.003, -0.02, -0.052, -0.084, -0.109, -0.124, -0.131, -0.129, -0.121, -0.108, -0.092,
        -0.073, -0.053, -0.032, -0.009, 0.015, 0.04, 0.067, 0.095, 0.123, 0.15, 0.176, 0.198,
        0.214, 0.224, 0.224, 0.214, 0.193, 0.16, 0.117, 0.066, 0.009, -0.048, -0.1, -0.14, -0.163,
        -0.16, -0.129, -0.065, 0.028, 0.141, 0.256, 0.344, 0.356,
    ],
    vtl_cm: 17.6,
};

/// Story 2018, Table IV, adult-female columns (length-warped derivation of the male model, not independent MRI). L(i) = 0.353 cm, VTL 15.53 cm.
pub const ADULT_FEMALE: TractBasis = TractBasis {
    omega: [
        0.63, 0.58, 0.56, 0.6, 0.76, 1.0, 1.17, 1.18, 1.13, 1.14, 1.17, 1.21, 1.24, 1.25, 1.25,
        1.27, 1.3, 1.34, 1.35, 1.35, 1.35, 1.35, 1.35, 1.34, 1.31, 1.3, 1.32, 1.4, 1.49, 1.58,
        1.66, 1.76, 1.87, 1.96, 2.03, 2.08, 2.07, 2.0, 1.86, 1.72, 1.6, 1.49, 1.39, 1.29,
    ],
    phi1: [
        0.003, -0.012, -0.026, -0.037, -0.048, -0.058, -0.069, -0.082, -0.096, -0.111, -0.126,
        -0.141, -0.154, -0.164, -0.17, -0.172, -0.168, -0.158, -0.141, -0.119, -0.09, -0.057,
        -0.019, 0.02, 0.061, 0.101, 0.139, 0.174, 0.203, 0.225, 0.24, 0.246, 0.244, 0.234, 0.216,
        0.192, 0.163, 0.132, 0.102, 0.074, 0.053, 0.04, 0.037, 0.047,
    ],
    phi2: [
        0.002, -0.018, -0.047, -0.074, -0.093, -0.104, -0.105, -0.101, -0.091, -0.077, -0.061,
        -0.044, -0.025, -0.005, 0.016, 0.039, 0.063, 0.088, 0.113, 0.137, 0.159, 0.178, 0.192,
        0.197, 0.197, 0.186, 0.166, 0.136, 0.097, 0.052, 0.002, -0.048, -0.094, -0.131, -0.155,
        -0.155, -0.138, -0.093, -0.023, 0.069, 0.172, 0.272, 0.34, 0.351,
    ],
    vtl_cm: 15.53,
};
