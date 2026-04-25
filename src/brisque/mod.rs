pub mod features;
pub mod model_data;
pub mod mscn;
pub mod stats;
pub mod svm;

use features::extract_brisque_features;
use svm::{compute_brisque_score, quality_label};

pub struct BrisqueResult {
    pub score: f64,
    pub quality: &'static str,
}

pub fn analyze(pixels: &[f64], width: usize, height: usize) -> BrisqueResult {
    let features = extract_brisque_features(pixels, width, height);
    let score = compute_brisque_score(&features);
    BrisqueResult {
        score,
        quality: quality_label(score),
    }
}
