//! Exposure detection via histogram analysis.
//!
//! Clipping thresholds:
//!   Under-exposed : > 5 % of pixels below 30  (shadow clipping)
//!   Over-exposed  : > 5 % of pixels above 225 (highlight clipping)

const SHADOW_CLIP: u8 = 30;
const HIGHLIGHT_CLIP: u8 = 225;
const CLIP_FRACTION: f64 = 0.05;

/// Returns one of: "underexposed", "overexposed", "normal".
pub fn detect_exposure(gray_u8: &[u8]) -> &'static str {
    if gray_u8.is_empty() {
        return "normal";
    }

    let n = gray_u8.len() as f64;
    let dark = gray_u8.iter().filter(|&&p| p < SHADOW_CLIP).count() as f64;
    let bright = gray_u8.iter().filter(|&&p| p > HIGHLIGHT_CLIP).count() as f64;

    if dark / n > CLIP_FRACTION {
        "underexposed"
    } else if bright / n > CLIP_FRACTION {
        "overexposed"
    } else {
        "normal"
    }
}
