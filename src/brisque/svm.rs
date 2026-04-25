//! BRISQUE scoring via a pre-trained RBF-kernel SVR.
//!
//! Model source: opencv/opencv_contrib `modules/quality/samples/`
//! (brisque_model_live.yml + brisque_range_live.yml)
//! Trained on the LIVE IQA database (EPS-SVR, C=1024, gamma=0.05).

use super::model_data::{ALPHAS, FEATURE_MAX, FEATURE_MIN, FEATURE_COUNT, GAMMA, RHO, SVS, SV_COUNT};

// ---------------------------------------------------------------------------
// Feature normalization
// ---------------------------------------------------------------------------

/// Scale each raw BRISQUE feature to [-1, 1] using the LIVE training-set range.
/// This must match the normalization applied when the model was trained.
fn normalize(features: &[f64; 36]) -> [f64; 36] {
    let mut out = [0.0f64; 36];
    for i in 0..FEATURE_COUNT {
        let range = FEATURE_MAX[i] - FEATURE_MIN[i];
        out[i] = if range < 1e-10 {
            0.0
        } else {
            2.0 * (features[i] - FEATURE_MIN[i]) / range - 1.0
        };
    }
    out
}

// ---------------------------------------------------------------------------
// SVR prediction
// ---------------------------------------------------------------------------

/// Compute the raw SVR decision value for a normalized feature vector.
///
/// decision(x) = Σ_i α_i · K(x, sv_i) + ρ
///
/// where K(x, sv) = exp(−γ · ‖x − sv‖²)
fn svr_predict(x: &[f64; 36]) -> f64 {
    let mut sum = 0.0f64;

    for i in 0..SV_COUNT {
        let sv = &SVS[i];
        let mut sq_dist = 0.0f64;
        for j in 0..FEATURE_COUNT {
            let d = x[j] - sv[j] as f64;
            sq_dist += d * d;
        }
        sum += ALPHAS[i] as f64 * (-GAMMA * sq_dist).exp();
    }

    sum + RHO
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compute the BRISQUE quality score in [0, 100] (lower = better).
///
/// Raw SVR output is clamped to [0, 100]; scores outside that range are
/// uncommon on natural images but can occur on synthetic or pathological inputs.
pub fn compute_brisque_score(features: &[f64; 36]) -> f64 {
    let x = normalize(features);
    svr_predict(&x).clamp(0.0, 100.0)
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
