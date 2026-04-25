/// Blur detection via Laplacian variance.
///
/// The 2-D Laplacian kernel [[0,1,0],[1,-4,1],[0,1,0]] measures local curvature.
/// Sharp images produce high variance in the response; blurry images have low variance.

const BLUR_THRESHOLD: f64 = 80.0;

/// Apply the Laplacian operator and return the per-pixel responses.
fn laplacian(pixels: &[f64], width: usize, height: usize) -> Vec<f64> {
    let mut out = vec![0.0f64; width * height];

    for row in 1..height.saturating_sub(1) {
        for col in 1..width.saturating_sub(1) {
            let idx = row * width + col;
            let resp = pixels[idx - width]       // top
                     + pixels[idx + width]       // bottom
                     + pixels[idx - 1]           // left
                     + pixels[idx + 1]           // right
                     - 4.0 * pixels[idx]; // centre
            out[idx] = resp;
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

/// Returns the Laplacian variance score (higher = sharper).
pub fn blur_score(pixels: &[f64], width: usize, height: usize) -> f64 {
    let lap = laplacian(pixels, width, height);
    variance(&lap)
}

/// True if the image is considered blurry.
pub fn is_blurry(pixels: &[f64], width: usize, height: usize) -> bool {
    blur_score(pixels, width, height) < BLUR_THRESHOLD
}
