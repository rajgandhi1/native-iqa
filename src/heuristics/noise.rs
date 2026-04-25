//! Noise estimation via variance in low-gradient (flat) regions.
//!
//! Flat regions are identified by a gradient magnitude below a threshold.
//! The variance of pixel values within those regions reflects sensor noise.

const GRADIENT_THRESHOLD: f64 = 8.0;

/// Compute gradient magnitude using finite differences.
fn gradient_magnitude(pixels: &[f64], width: usize, height: usize) -> Vec<f64> {
    let mut out = vec![0.0f64; width * height];

    for row in 1..height.saturating_sub(1) {
        for col in 1..width.saturating_sub(1) {
            let gx = pixels[row * width + col + 1] - pixels[row * width + col - 1];
            let gy = pixels[(row + 1) * width + col] - pixels[(row - 1) * width + col];
            out[row * width + col] = (gx * gx + gy * gy).sqrt();
        }
    }
    out
}

fn variance(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let n = data.len() as f64;
    let mean = data.iter().sum::<f64>() / n;
    data.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n
}

/// Returns "low", "medium", or "high".
pub fn estimate_noise(pixels: &[f64], width: usize, height: usize) -> &'static str {
    let grad = gradient_magnitude(pixels, width, height);

    let flat_vals: Vec<f64> = pixels
        .iter()
        .zip(grad.iter())
        .filter(|(_, &g)| g < GRADIENT_THRESHOLD)
        .map(|(&p, _)| p)
        .collect();

    if flat_vals.len() < 64 {
        // Not enough flat area to judge
        return "low";
    }

    let var = variance(&flat_vals);

    if var < 6.0 {
        "low"
    } else if var < 25.0 {
        "medium"
    } else {
        "high"
    }
}
