/// Image decoding, resizing, and dispatch to BRISQUE + heuristics.
use image::{DynamicImage, GenericImageView, GrayImage};

use crate::brisque;
use crate::heuristics;

/// Maximum dimension (width or height) before resizing.
/// Keeps processing fast while retaining enough detail for BRISQUE.
const MAX_DIM: u32 = 512;

/// Minimum dimension needed to run BRISQUE reliably.
const MIN_DIM: u32 = 16;

pub struct AnalysisResult {
    pub score: f64,
    pub quality: String,
    pub is_blurry: bool,
    pub exposure: String,
    pub noise_level: String,
    pub warnings: Vec<String>,
}

// ---------------------------------------------------------------------------
// Decode
// ---------------------------------------------------------------------------

fn decode(data: &[u8]) -> Result<DynamicImage, String> {
    image::load_from_memory(data).map_err(|e| format!("Failed to decode image: {e}"))
}

// ---------------------------------------------------------------------------
// Resize (preserving aspect ratio)
// ---------------------------------------------------------------------------

fn maybe_resize(img: DynamicImage) -> DynamicImage {
    let (w, h) = img.dimensions();
    let largest = w.max(h);
    if largest <= MAX_DIM {
        return img;
    }
    let scale = MAX_DIM as f64 / largest as f64;
    let new_w = ((w as f64 * scale) as u32).max(1);
    let new_h = ((h as f64 * scale) as u32).max(1);
    img.resize_exact(new_w, new_h, image::imageops::FilterType::Triangle)
}

// ---------------------------------------------------------------------------
// Grayscale conversion
// ---------------------------------------------------------------------------

fn to_gray(img: &DynamicImage) -> GrayImage {
    img.to_luma8()
}

fn gray_to_f64(gray: &GrayImage) -> Vec<f64> {
    gray.pixels().map(|p| p.0[0] as f64).collect()
}

// ---------------------------------------------------------------------------
// Main entry points
// ---------------------------------------------------------------------------

/// Returns only the BRISQUE score, skipping all heuristics.
pub fn score_only(data: &[u8]) -> Result<f64, String> {
    let img = decode(data)?;

    let (w, h) = img.dimensions();
    if w < MIN_DIM || h < MIN_DIM {
        return Err(format!(
            "Image too small ({}×{}). Minimum is {}×{}.",
            w, h, MIN_DIM, MIN_DIM
        ));
    }

    let img = maybe_resize(img);
    let gray = to_gray(&img);

    let (width, height) = gray.dimensions();
    let pixels_f64 = gray_to_f64(&gray);

    Ok(brisque::analyze(&pixels_f64, width as usize, height as usize).score)
}

pub fn analyze(data: &[u8]) -> Result<AnalysisResult, String> {
    let img = decode(data)?;

    let (w, h) = img.dimensions();
    if w < MIN_DIM || h < MIN_DIM {
        return Err(format!(
            "Image too small ({}×{}). Minimum is {}×{}.",
            w, h, MIN_DIM, MIN_DIM
        ));
    }

    let img = maybe_resize(img);
    let gray = to_gray(&img);

    let (width, height) = gray.dimensions();
    let pixels_u8: Vec<u8> = gray.pixels().map(|p| p.0[0]).collect();
    let pixels_f64 = gray_to_f64(&gray);

    // BRISQUE
    let brisque_res = brisque::analyze(&pixels_f64, width as usize, height as usize);

    // Heuristics
    let heuristics_res = heuristics::run(&pixels_f64, &pixels_u8, width as usize, height as usize);

    Ok(AnalysisResult {
        score: brisque_res.score,
        quality: brisque_res.quality.to_string(),
        is_blurry: heuristics_res.is_blurry,
        exposure: heuristics_res.exposure.to_string(),
        noise_level: heuristics_res.noise_level.to_string(),
        warnings: heuristics_res.warnings,
    })
}
