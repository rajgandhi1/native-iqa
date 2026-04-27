/// BRISQUE feature extraction.
///
/// Extracts 36 features from an image at two scales:
///   Scale 1 (original):  18 features
///   Scale 2 (half-size): 18 features
///
/// At each scale, 18 features are:
///   2  from AGGD fit to MSCN coefficients: [alpha, (σ_l² + σ_r²)/2]
///   4  from AGGD fit to each of 4 pairwise-product maps (H, V, D1, D2):
///      [alpha, eta, σ_l², σ_r²]   — variances, matching OpenCV qualitybrisque.cpp
use super::mscn::compute_mscn;
use super::stats::{fit_aggd};

/// Extract 18 BRISQUE features from one scale.
/// Mirrors OpenCV's ComputeBrisqueFeature() in qualitybrisque.cpp exactly.
fn extract_scale_features(pixels: &[f64], width: usize, height: usize) -> [f64; 18] {
    let mscn = compute_mscn(pixels, width, height);

    // --- AGGD fit to MSCN (OpenCV uses AGGD for the MSCN itself) ---
    let (alpha, _eta, sigma_l, sigma_r) = fit_aggd(&mscn);
    // feat[1] = (σ_l² + σ_r²) / 2  (matches OpenCV)
    let sigma_sq = (sigma_l * sigma_l + sigma_r * sigma_r) / 2.0;

    // --- Pairwise products ---
    // Matches OpenCV shifts: {0,1}, {1,0}, {1,1}, {-1,1}  (H, V, D1, D2)
    // OpenCV zeros out out-of-bounds positions; since those zeros are excluded
    // from the AGGD fit (only |pt| > 0 are counted), trimming produces the same result.
    let mscn2d: Vec<&[f64]> = mscn.chunks(width).collect();

    let mut h = Vec::with_capacity((width - 1) * height);
    let mut v = Vec::with_capacity(width * (height - 1));
    let mut d1 = Vec::with_capacity((width - 1) * (height - 1));
    let mut d2 = Vec::with_capacity((width - 1) * (height - 1));

    for row in 0..height {
        for col in 0..width {
            let val = mscn2d[row][col];
            if col + 1 < width {
                h.push(val * mscn2d[row][col + 1]);
            }
            if row + 1 < height {
                v.push(val * mscn2d[row + 1][col]);
            }
            if row + 1 < height && col + 1 < width {
                d1.push(val * mscn2d[row + 1][col + 1]);
            }
            if row + 1 < height && col > 0 {
                d2.push(val * mscn2d[row + 1][col - 1]);
            }
        }
    }

    // AGGD returns (alpha, eta, sigma_l, sigma_r).
    // OpenCV stores variances: push lsigma^2 and rsigma^2.
    let (h_alpha, h_eta, h_sl, h_sr) = fit_aggd(&h);
    let (v_alpha, v_eta, v_sl, v_sr) = fit_aggd(&v);
    let (d1_alpha, d1_eta, d1_sl, d1_sr) = fit_aggd(&d1);
    let (d2_alpha, d2_eta, d2_sl, d2_sr) = fit_aggd(&d2);

    [
        // MSCN: [alpha, (σ_l² + σ_r²)/2]
        alpha, sigma_sq,
        // H:  [alpha, eta, σ_l², σ_r²]
        h_alpha,  h_eta,  h_sl * h_sl,  h_sr * h_sr,
        // V:  [alpha, eta, σ_l², σ_r²]
        v_alpha,  v_eta,  v_sl * v_sl,  v_sr * v_sr,
        // D1: [alpha, eta, σ_l², σ_r²]
        d1_alpha, d1_eta, d1_sl * d1_sl, d1_sr * d1_sr,
        // D2: [alpha, eta, σ_l², σ_r²]
        d2_alpha, d2_eta, d2_sl * d2_sl, d2_sr * d2_sr,
    ]
}

/// Bicubic downsampling to half resolution (matches OpenCV INTER_CUBIC).
///
/// Uses a 1-D bicubic kernel applied separably (horizontal then vertical).
/// Bicubic kernel: h(t) = (a+2)|t|³ - (a+3)|t|² + 1 for |t| ≤ 1
///                       a|t|³ - 5a|t|² + 8a|t| - 4a for 1 < |t| ≤ 2
///                       0 otherwise,  with a = -0.75.
pub fn downsample(pixels: &[f64], width: usize, height: usize) -> (Vec<f64>, usize, usize) {
    let new_w = width / 2;
    let new_h = height / 2;

    #[inline]
    fn cubic_weight(t: f64) -> f64 {
        const A: f64 = -0.75;
        let t = t.abs();
        if t <= 1.0 {
            (A + 2.0) * t * t * t - (A + 3.0) * t * t + 1.0
        } else if t <= 2.0 {
            A * t * t * t - 5.0 * A * t * t + 8.0 * A * t - 4.0 * A
        } else {
            0.0
        }
    }

    /// Clamp index to valid range.
    #[inline]
    fn clamp_idx(i: i32, size: usize) -> usize {
        i.clamp(0, size as i32 - 1) as usize
    }

    let src = pixels;

    // Horizontal pass: resample columns from width → new_w, keep all rows.
    let mut tmp = vec![0.0f64; new_w * height];
    let scale_x = width as f64 / new_w as f64;
    for row in 0..height {
        for col in 0..new_w {
            // Source position (centre of dst pixel in src space)
            let src_x = (col as f64 + 0.5) * scale_x - 0.5;
            let ix = src_x.floor() as i32;
            let mut acc = 0.0f64;
            for k in -1i32..=2 {
                let w = cubic_weight(src_x - (ix + k) as f64);
                let sc = clamp_idx(ix + k, width);
                acc += w * src[row * width + sc];
            }
            tmp[row * new_w + col] = acc;
        }
    }

    // Vertical pass: resample rows from height → new_h, keep new_w columns.
    let mut out = vec![0.0f64; new_w * new_h];
    let scale_y = height as f64 / new_h as f64;
    for row in 0..new_h {
        let src_y = (row as f64 + 0.5) * scale_y - 0.5;
        let iy = src_y.floor() as i32;
        for col in 0..new_w {
            let mut acc = 0.0f64;
            for k in -1i32..=2 {
                let w = cubic_weight(src_y - (iy + k) as f64);
                let sr = clamp_idx(iy + k, height);
                acc += w * tmp[sr * new_w + col];
            }
            out[row * new_w + col] = acc;
        }
    }

    (out, new_w, new_h)
}

/// Extract all 36 BRISQUE features across two scales.
pub fn extract_brisque_features(pixels: &[f64], width: usize, height: usize) -> [f64; 36] {
    let f1 = extract_scale_features(pixels, width, height);

    // Scale 2: half-resolution via bicubic downsampling (matches OpenCV INTER_CUBIC)
    let (px2, w2, h2) = downsample(pixels, width, height);
    let f2 = if w2 >= 7 && h2 >= 7 {
        extract_scale_features(&px2, w2, h2)
    } else {
        f1
    };

    let mut features = [0.0f64; 36];
    features[..18].copy_from_slice(&f1);
    features[18..].copy_from_slice(&f2);
    features
}
