/// BRISQUE feature extraction.
///
/// Extracts 36 features from an image at two scales:
///   Scale 1 (original):  18 features
///   Scale 2 (half-size): 18 features
///
/// At each scale, 18 features are:
///   2  from GGD fit to MSCN coefficients
///   4  from AGGD fit to each of 4 pairwise-product maps (H, V, D1, D2)
///   → 2 + 4*4 = 18
use super::mscn::compute_mscn;
use super::stats::{fit_aggd, fit_ggd};

/// Extract 18 BRISQUE features from one scale.
///
/// `pixels` must be a row-major f64 grayscale image (values 0–255).
fn extract_scale_features(pixels: &[f64], width: usize, height: usize) -> [f64; 18] {
    let mscn = compute_mscn(pixels, width, height);

    // --- GGD fit to MSCN ---
    let (alpha, sigma_sq) = fit_ggd(&mscn);

    // --- Pairwise products ---
    // H:  mscn[i,j] * mscn[i, j+1]
    // V:  mscn[i,j] * mscn[i+1, j]
    // D1: mscn[i,j] * mscn[i+1, j+1]
    // D2: mscn[i,j] * mscn[i+1, j-1]
    let mut h = Vec::with_capacity((width - 1) * height);
    let mut v = Vec::with_capacity(width * (height - 1));
    let mut d1 = Vec::with_capacity((width - 1) * (height - 1));
    let mut d2 = Vec::with_capacity((width - 1) * (height - 1));

    for row in 0..height {
        for col in 0..width {
            let idx = row * width + col;
            let val = mscn[idx];

            // Horizontal
            if col + 1 < width {
                h.push(val * mscn[row * width + col + 1]);
            }
            // Vertical
            if row + 1 < height {
                v.push(val * mscn[(row + 1) * width + col]);
            }
            // Diagonal (top-left → bottom-right)
            if row + 1 < height && col + 1 < width {
                d1.push(val * mscn[(row + 1) * width + col + 1]);
            }
            // Diagonal (top-right → bottom-left)
            if row + 1 < height && col > 0 {
                d2.push(val * mscn[(row + 1) * width + col - 1]);
            }
        }
    }

    let (h_shape, h_eta, h_sl, h_sr) = fit_aggd(&h);
    let (v_shape, v_eta, v_sl, v_sr) = fit_aggd(&v);
    let (d1_shape, d1_eta, d1_sl, d1_sr) = fit_aggd(&d1);
    let (d2_shape, d2_eta, d2_sl, d2_sr) = fit_aggd(&d2);

    [
        // GGD
        alpha, sigma_sq, // H
        h_shape, h_eta, h_sl, h_sr, // V
        v_shape, v_eta, v_sl, v_sr, // D1
        d1_shape, d1_eta, d1_sl, d1_sr, // D2
        d2_shape, d2_eta, d2_sl, d2_sr,
    ]
}

/// Downsample a grayscale image to half resolution using simple 2×2 averaging.
pub fn downsample(pixels: &[f64], width: usize, height: usize) -> (Vec<f64>, usize, usize) {
    let new_w = width / 2;
    let new_h = height / 2;
    let mut out = vec![0.0f64; new_w * new_h];

    for row in 0..new_h {
        for col in 0..new_w {
            let r0 = 2 * row;
            let c0 = 2 * col;
            out[row * new_w + col] = 0.25
                * (pixels[r0 * width + c0]
                    + pixels[r0 * width + c0 + 1]
                    + pixels[(r0 + 1) * width + c0]
                    + pixels[(r0 + 1) * width + c0 + 1]);
        }
    }

    (out, new_w, new_h)
}

/// Extract all 36 BRISQUE features across two scales.
pub fn extract_brisque_features(pixels: &[f64], width: usize, height: usize) -> [f64; 36] {
    let f1 = extract_scale_features(pixels, width, height);

    // Scale 2: half-resolution
    let (px2, w2, h2) = downsample(pixels, width, height);
    let f2 = if w2 >= 7 && h2 >= 7 {
        extract_scale_features(&px2, w2, h2)
    } else {
        f1 // degenerate: image too small
    };

    let mut features = [0.0f64; 36];
    features[..18].copy_from_slice(&f1);
    features[18..].copy_from_slice(&f2);
    features
}
