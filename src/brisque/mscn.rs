/// MSCN (Mean Subtracted Contrast Normalized) coefficient computation.
///
/// MSCN(i,j) = (I(i,j) - μ(i,j)) / (σ(i,j) + C)
///
/// Where μ and σ are computed with a 7×7 Gaussian kernel (σ = 7/6).

const KERNEL_SIZE: usize = 7;
const KERNEL_SIGMA: f64 = 7.0 / 6.0;
const C: f64 = 1.0;

/// Build a 1-D normalised Gaussian kernel.
fn gaussian_kernel_1d() -> [f64; KERNEL_SIZE] {
    let half = (KERNEL_SIZE / 2) as f64;
    let two_sigma_sq = 2.0 * KERNEL_SIGMA * KERNEL_SIGMA;
    let mut k = [0.0f64; KERNEL_SIZE];
    for (i, v) in k.iter_mut().enumerate() {
        let x = i as f64 - half;
        *v = (-x * x / two_sigma_sq).exp();
    }
    let sum: f64 = k.iter().sum();
    k.iter_mut().for_each(|v| *v /= sum);
    k
}

/// Reflect-101 boundary extension: mirror without repeating the edge pixel.
/// Index -1 → 1, -2 → 2 … index (N) → N-2, (N+1) → N-3 …
#[inline(always)]
fn reflect(i: i32, size: i32) -> usize {
    let i = if i < 0 {
        -i
    } else if i >= size {
        2 * size - i - 2
    } else {
        i
    };
    i.max(0) as usize
}

/// 2-D separable Gaussian filter (horizontal then vertical pass).
pub fn gaussian_filter(src: &[f64], width: usize, height: usize) -> Vec<f64> {
    let k = gaussian_kernel_1d();
    let half = KERNEL_SIZE as i32 / 2;

    // --- horizontal pass ---
    let mut tmp = vec![0.0f64; width * height];
    for row in 0..height {
        for col in 0..width {
            let mut acc = 0.0f64;
            for (ki, &kv) in k.iter().enumerate() {
                let sc = reflect(col as i32 + ki as i32 - half, width as i32);
                acc += kv * src[row * width + sc];
            }
            tmp[row * width + col] = acc;
        }
    }

    // --- vertical pass ---
    let mut out = vec![0.0f64; width * height];
    for row in 0..height {
        for col in 0..width {
            let mut acc = 0.0f64;
            for (ki, &kv) in k.iter().enumerate() {
                let sr = reflect(row as i32 + ki as i32 - half, height as i32);
                acc += kv * tmp[sr * width + col];
            }
            out[row * width + col] = acc;
        }
    }
    out
}

/// Compute MSCN coefficients from a float grayscale image (values in [0, 255]).
///
/// Returns the MSCN map as a flat Vec<f64> in row-major order.
pub fn compute_mscn(pixels: &[f64], width: usize, height: usize) -> Vec<f64> {
    // Local mean
    let mu = gaussian_filter(pixels, width, height);

    // Local variance: E[(x - μ)²] via another Gaussian pass
    let diff_sq: Vec<f64> = pixels
        .iter()
        .zip(mu.iter())
        .map(|(p, m)| (p - m).powi(2))
        .collect();
    let sigma_sq = gaussian_filter(&diff_sq, width, height);

    // MSCN = (x - μ) / (σ + C)
    pixels
        .iter()
        .zip(mu.iter())
        .zip(sigma_sq.iter())
        .map(|((p, m), s)| (p - m) / (s.sqrt() + C))
        .collect()
}
