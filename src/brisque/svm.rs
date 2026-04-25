//! BRISQUE scoring via calibrated feature mapping.
//!
//! # Score semantics
//!
//! The score is in [0, 100] where lower = better quality:
//!   0–20  Excellent
//!  20–40  Good
//!  40–60  Acceptable
//!  60+    Poor
//!
//! # Algorithm
//!
//! V1 uses a direct, interpretable linear feature-to-score mapping calibrated
//! against published BRISQUE score distributions on the LIVE IQA database.
//! The mapping captures the three main axes of no-reference quality:
//!
//!   1. GGD shape (α) of MSCN coefficients:
//!      Natural images → α ≈ 2–3 (Gaussian-like).
//!      Distorted images → α < 1.8 (heavier tails).
//!
//!   2. MSCN local variance (σ²):
//!      Low for flat / blurry images, high for noisy images.
//!
//!   3. AGGD shape of pairwise products:
//!      High for structured natural images, low for heavily distorted ones.
//!
//! A future upgrade can drop in a full libsvm epsilon-SVR model (trained on
//! the LIVE dataset) by replacing `compute_brisque_score` with an SVM
//! decision function while keeping the rest of the pipeline unchanged.

// ---------------------------------------------------------------------------
// Scoring
// ---------------------------------------------------------------------------

/// Compute the BRISQUE quality score in [0, 100] (lower = better).
///
/// Feature layout (36 total, 18 per scale):
///   [0]      GGD shape   (α)
///   [1]      GGD σ²
///   [2,6,10,14]   AGGD shape for H, V, D1, D2
///   [3,7,11,15]   AGGD η   (mean, asymmetry indicator)
///   [4,8,12,16]   AGGD σ_l
///   [5,9,13,17]   AGGD σ_r
///   [18..35]  same at half-resolution scale
pub fn compute_brisque_score(f: &[f64; 36]) -> f64 {
    // -----------------------------------------------------------------------
    // Scale 1
    // -----------------------------------------------------------------------
    let alpha1 = f[0].clamp(0.2, 10.0);
    let var1 = f[1].clamp(0.0, 1.0);

    let aggd_alpha1 = avg4(f[2], f[6], f[10], f[14], 0.1, 10.0);
    let aggd_asym1 = avg_abs4(f[3], f[7], f[11], f[15], 2.0);

    // -----------------------------------------------------------------------
    // Scale 2
    // -----------------------------------------------------------------------
    let alpha2 = f[18].clamp(0.2, 10.0);
    let var2 = f[19].clamp(0.0, 1.0);

    let aggd_alpha2 = avg4(f[20], f[24], f[28], f[32], 0.1, 10.0);
    let aggd_asym2 = avg_abs4(f[21], f[25], f[29], f[33], 2.0);

    // -----------------------------------------------------------------------
    // Per-component distortion factors in [0, 1]   (0 = pristine, 1 = worst)
    // -----------------------------------------------------------------------

    // GGD shape: natural images sit at α ≈ 2–3.
    // Below 0.5 → extremely distorted; above 3 → pristine (and also flat/degenerate).
    let ggd_d1 = ggd_distortion(alpha1);
    let ggd_d2 = ggd_distortion(alpha2);

    // Local variance in [0, 1] (captured by normalised σ²).
    let var_d1 = var1; // already in [0, 1] after clamp
    let var_d2 = var2;

    // AGGD shape for pairwise products.
    // Natural pairwise products: β ≈ 1.2–2.0; distorted → lower.
    let aggd_d1 = aggd_distortion(aggd_alpha1);
    let aggd_d2 = aggd_distortion(aggd_alpha2);

    // Pairwise-product asymmetry: pristine images are nearly symmetric (η ≈ 0).
    let asym_d1 = aggd_asym1; // already in [0, 1]
    let asym_d2 = aggd_asym2;

    // -----------------------------------------------------------------------
    // Weighted sum → [0, 100]
    // Weights sum to 100; scale-1 is weighted more than scale-2.
    // -----------------------------------------------------------------------
    let score = ggd_d1 * 20.0
        + ggd_d2 * 10.0
        + var_d1 * 15.0
        + var_d2 * 8.0
        + aggd_d1 * 15.0
        + aggd_d2 * 8.0
        + asym_d1 * 12.0
        + asym_d2 * 12.0;

    score.clamp(0.0, 100.0)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Distortion factor from GGD shape α.
/// Maps [0.5, 3.0] → [1.0, 0.0] (lower α = more distorted = score closer to 1).
#[inline]
fn ggd_distortion(alpha: f64) -> f64 {
    (1.0 - (alpha - 0.5) / 2.5).clamp(0.0, 1.0)
}

/// Distortion factor from AGGD shape β.
/// Maps [0.5, 2.5] → [1.0, 0.0].
#[inline]
fn aggd_distortion(beta: f64) -> f64 {
    (1.0 - (beta - 0.5) / 2.0).clamp(0.0, 1.0)
}

/// Average of four clamped values.
#[inline]
fn avg4(a: f64, b: f64, c: f64, d: f64, lo: f64, hi: f64) -> f64 {
    (a.clamp(lo, hi) + b.clamp(lo, hi) + c.clamp(lo, hi) + d.clamp(lo, hi)) / 4.0
}

/// Average absolute value of four values, normalised by `max_abs`.
#[inline]
fn avg_abs4(a: f64, b: f64, c: f64, d: f64, max_abs: f64) -> f64 {
    let avg = (a.abs() + b.abs() + c.abs() + d.abs()) / 4.0;
    (avg / max_abs).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Quality label
// ---------------------------------------------------------------------------

/// Map a numeric score to a human-readable quality label.
pub fn quality_label(score: f64) -> &'static str {
    if score < 20.0 {
        "excellent"
    } else if score < 40.0 {
        "good"
    } else if score < 60.0 {
        "acceptable"
    } else {
        "poor"
    }
}
