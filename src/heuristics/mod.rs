pub mod blur;
pub mod exposure;
pub mod noise;

pub struct HeuristicsResult {
    pub is_blurry: bool,
    pub exposure: &'static str,
    pub noise_level: &'static str,
    pub warnings: Vec<String>,
}

pub fn run(pixels_f64: &[f64], pixels_u8: &[u8], width: usize, height: usize) -> HeuristicsResult {
    let is_blurry = blur::is_blurry(pixels_f64, width, height);
    let exposure = exposure::detect_exposure(pixels_u8);
    let noise_level = noise::estimate_noise(pixels_f64, width, height);

    let mut warnings = Vec::new();
    if is_blurry {
        warnings.push("Image appears blurry".to_string());
    }
    if exposure != "normal" {
        warnings.push(format!("Image is {}", exposure));
    }
    if noise_level == "high" {
        warnings.push("High noise level detected".to_string());
    }

    HeuristicsResult {
        is_blurry,
        exposure,
        noise_level,
        warnings,
    }
}
