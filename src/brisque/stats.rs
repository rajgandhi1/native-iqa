//! Statistical distribution fitting for BRISQUE feature extraction.
//!
//! Implements:
//!   - GGD  (Generalized Gaussian Distribution) fitting
//!   - AGGD (Asymmetric GGD) fitting
//!
//! Both use a binary-search approach to invert the moment-ratio function
//! r(α) = Γ(2/α)² / (Γ(1/α)·Γ(3/α)).

// ---------------------------------------------------------------------------
// Gamma function (Lanczos approximation, accurate to ~15 sig. figs)
// ---------------------------------------------------------------------------

pub fn gamma_fn(x: f64) -> f64 {
    if x < 0.5 {
        std::f64::consts::PI / ((std::f64::consts::PI * x).sin() * gamma_fn(1.0 - x))
    } else {
        // Lanczos coefficients (g = 7)
        const G: f64 = 7.0;
        const C: [f64; 9] = [
            0.999_999_999_999_809_9,
            676.5203681218851,
            -1259.1392167224028,
            771.323_428_777_653_1,
            -176.615_029_162_140_6,
            12.507343278686905,
            -0.13857109526572012,
            9.984_369_578_019_572e-6,
            1.5056327351493116e-7,
        ];

        let x = x - 1.0;
        let t = x + G + 0.5;
        let mut sum = C[0];
        for (i, &ci) in C[1..].iter().enumerate() {
            sum += ci / (x + (i + 1) as f64);
        }
        (2.0 * std::f64::consts::PI).sqrt() * t.powf(x + 0.5) * (-t).exp() * sum
    }
}

// ---------------------------------------------------------------------------
// Shape-parameter inversion via binary search
// ---------------------------------------------------------------------------

/// r(α) = Γ(2/α)² / (Γ(1/α)·Γ(3/α))
#[inline]
fn ggd_r(alpha: f64) -> f64 {
    let g2 = gamma_fn(2.0 / alpha);
    let g1 = gamma_fn(1.0 / alpha);
    let g3 = gamma_fn(3.0 / alpha);
    (g2 * g2) / (g1 * g3)
}

/// Find α ∈ [0.2, 10] such that ggd_r(α) ≈ ratio.
///
/// ggd_r is monotonically increasing in α, so binary search works.
fn find_shape(ratio: f64) -> f64 {
    let ratio = ratio.clamp(ggd_r(0.2) + 1e-9, ggd_r(10.0) - 1e-9);
    let mut lo = 0.2f64;
    let mut hi = 10.0f64;
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if ggd_r(mid) < ratio {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

// ---------------------------------------------------------------------------
// GGD fit  →  (alpha, sigma_sq)
// ---------------------------------------------------------------------------

/// Fit a zero-mean GGD to `data`.
///
/// Returns `(α, σ²)` where:
///   α  = shape parameter  (Gaussian ≈ 2.0; heavier tails → α < 2)
///   σ² = variance
#[allow(dead_code)]
pub fn fit_ggd(data: &[f64]) -> (f64, f64) {
    if data.is_empty() {
        return (2.0, 0.0);
    }
    let n = data.len() as f64;

    let mean_abs: f64 = data.iter().map(|x| x.abs()).sum::<f64>() / n;
    let mean_sq: f64 = data.iter().map(|x| x * x).sum::<f64>() / n;

    if mean_sq < 1e-12 {
        return (2.0, 0.0);
    }

    let r_hat = (mean_abs * mean_abs) / mean_sq;
    let alpha = find_shape(r_hat);

    (alpha, mean_sq)
}

// ---------------------------------------------------------------------------
// AGGD fit  →  (alpha, mean, left_std, right_std)
// ---------------------------------------------------------------------------

/// Fit an Asymmetric GGD to `data` (which may have a non-zero mean).
///
/// Returns `(α, η, σ_l, σ_r)` where:
///   α   = shape parameter
///   η   = distribution mean
///   σ_l = left-side scale
///   σ_r = right-side scale
pub fn fit_aggd(data: &[f64]) -> (f64, f64, f64, f64) {
    if data.is_empty() {
        return (2.0, 0.0, 1.0, 1.0);
    }
    let n = data.len() as f64;

    // Split into strictly negative / strictly positive (zeros excluded from both halves).
    // Matches OpenCV's AGGDfit which checks pt > 0 and pt < 0 separately,
    // leaving zero-valued pixels uncounted in sigma_l and sigma_r.
    let left_sq: f64 = data.iter().filter(|&&x| x < 0.0).map(|x| x * x).sum();
    let left_n = data.iter().filter(|&&x| x < 0.0).count() as f64;

    let right_sq: f64 = data.iter().filter(|&&x| x > 0.0).map(|x| x * x).sum();
    let right_n = data.iter().filter(|&&x| x > 0.0).count() as f64;

    let sigma_l = if left_n > 0.0 {
        (left_sq / left_n).sqrt()
    } else {
        1e-6
    };
    let sigma_r = if right_n > 0.0 {
        (right_sq / right_n).sqrt()
    } else {
        1e-6
    };

    // Compute overall shape ratio (asymmetry-corrected)
    let gamma_hat = sigma_l / sigma_r.max(1e-10);
    let mean_abs: f64 = data.iter().map(|x| x.abs()).sum::<f64>() / n;
    let mean_sq: f64 = data.iter().map(|x| x * x).sum::<f64>() / n;

    if mean_sq < 1e-12 {
        return (2.0, 0.0, sigma_l, sigma_r);
    }

    let r_hat = (mean_abs * mean_abs) / mean_sq;
    let r_hat_norm =
        r_hat * (gamma_hat.powi(3) + 1.0) * (gamma_hat + 1.0) / (gamma_hat.powi(2) + 1.0).powi(2);

    let alpha = find_shape(r_hat_norm);

    // Mean parameter η = (σ_r − σ_l) · Γ(2/α) / √(Γ(1/α)·Γ(3/α))
    // This is derived from the AGGD beta parameterisation: β = σ · √(Γ(1/α)/Γ(3/α))
    // and η = (β_r − β_l) · Γ(2/α) / Γ(1/α).
    let g1 = gamma_fn(1.0 / alpha);
    let g2 = gamma_fn(2.0 / alpha);
    let g3 = gamma_fn(3.0 / alpha);
    let eta = (sigma_r - sigma_l) * g2 / (g1 * g3).sqrt();

    (alpha, eta, sigma_l, sigma_r)
}
