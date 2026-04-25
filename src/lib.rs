#![deny(clippy::all)]
#![allow(private_interfaces)]

mod brisque;
mod heuristics;
mod pipeline;

use napi::bindgen_prelude::*;
use napi::Task;
use napi_derive::napi;
use rayon::prelude::*;

// ---------------------------------------------------------------------------
// Exported types
// ---------------------------------------------------------------------------

#[napi(object)]
pub struct IqaResult {
    /// BRISQUE-based quality score in [0, 100].  Lower = better.
    ///   0–20  Excellent
    ///  20–40  Good
    ///  40–60  Acceptable
    ///  60+    Poor
    pub score: f64,
    /// Human-readable quality label: "excellent" | "good" | "acceptable" | "poor"
    pub quality: String,
    /// True when Laplacian variance indicates motion/focus blur.
    pub is_blurry: bool,
    /// Exposure assessment: "normal" | "underexposed" | "overexposed"
    pub exposure: String,
    /// Noise estimate: "low" | "medium" | "high"
    pub noise_level: String,
    /// List of human-readable warnings (empty for clean images).
    pub warnings: Vec<String>,
}

#[napi(object)]
pub struct ValidationOptions {
    /// Reject images with score above this threshold (lower = stricter).
    pub min_score: Option<f64>,
    /// Reject blurry images when true.
    pub reject_blurry: Option<bool>,
    /// Reject non-normal exposures when true.
    pub reject_bad_exposure: Option<bool>,
}

#[napi(object)]
pub struct ValidationResult {
    pub passed: bool,
    pub score: f64,
    pub quality: String,
    pub failures: Vec<String>,
}

// ---------------------------------------------------------------------------
// Async Task wrappers
// ---------------------------------------------------------------------------

// --- analyze ---

pub(crate) struct AnalyzeTask {
    data: Vec<u8>,
}

impl Task for AnalyzeTask {
    type Output = pipeline::AnalysisResult;
    type JsValue = IqaResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        pipeline::analyze(&self.data).map_err(napi::Error::from_reason)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(IqaResult {
            score: output.score,
            quality: output.quality,
            is_blurry: output.is_blurry,
            exposure: output.exposure,
            noise_level: output.noise_level,
            warnings: output.warnings,
        })
    }
}

// --- quickScore ---

pub(crate) struct QuickScoreTask {
    data: Vec<u8>,
}

impl Task for QuickScoreTask {
    type Output = f64;
    type JsValue = f64;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        pipeline::analyze(&self.data)
            .map(|r| r.score)
            .map_err(napi::Error::from_reason)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output)
    }
}

// --- validate ---

pub(crate) struct ValidateTask {
    data: Vec<u8>,
    options: ValidationOptions,
}

impl Task for ValidateTask {
    type Output = (pipeline::AnalysisResult, ValidationOptions);
    type JsValue = ValidationResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let result = pipeline::analyze(&self.data).map_err(napi::Error::from_reason)?;

        // Clone options for use in resolve
        let opts = ValidationOptions {
            min_score: self.options.min_score,
            reject_blurry: self.options.reject_blurry,
            reject_bad_exposure: self.options.reject_bad_exposure,
        };
        Ok((result, opts))
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        let (result, opts) = output;
        let mut failures = Vec::new();

        let min_score = opts.min_score.unwrap_or(60.0);
        if result.score > min_score {
            failures.push(format!(
                "Score {:.1} exceeds threshold {:.1}",
                result.score, min_score
            ));
        }

        if opts.reject_blurry.unwrap_or(false) && result.is_blurry {
            failures.push("Image is blurry".to_string());
        }

        if opts.reject_bad_exposure.unwrap_or(false) && result.exposure != "normal" {
            failures.push(format!("Bad exposure: {}", result.exposure));
        }

        Ok(ValidationResult {
            passed: failures.is_empty(),
            score: result.score,
            quality: result.quality,
            failures,
        })
    }
}

// --- scoreBatch ---

pub(crate) struct BatchTask {
    items: Vec<Vec<u8>>,
}

impl Task for BatchTask {
    type Output = Vec<pipeline::AnalysisResult>;
    type JsValue = Vec<IqaResult>;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        // Process each image; propagate first error.
        self.items
            .par_iter()
            .map(|d| pipeline::analyze(d).map_err(napi::Error::from_reason))
            .collect()
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(output
            .into_iter()
            .map(|r| IqaResult {
                score: r.score,
                quality: r.quality,
                is_blurry: r.is_blurry,
                exposure: r.exposure,
                noise_level: r.noise_level,
                warnings: r.warnings,
            })
            .collect())
    }
}

// ---------------------------------------------------------------------------
// Exported API
// ---------------------------------------------------------------------------

/// Full quality analysis. Returns a Promise<IqaResult>.
#[napi]
pub fn analyze(buffer: Buffer) -> AsyncTask<AnalyzeTask> {
    AsyncTask::new(AnalyzeTask {
        data: buffer.to_vec(),
    })
}

/// Returns only the numeric BRISQUE score. Slightly faster than analyze().
#[napi]
pub fn quick_score(buffer: Buffer) -> AsyncTask<QuickScoreTask> {
    AsyncTask::new(QuickScoreTask {
        data: buffer.to_vec(),
    })
}

/// Validates an image against configurable quality thresholds.
#[napi]
pub fn validate(buffer: Buffer, options: Option<ValidationOptions>) -> AsyncTask<ValidateTask> {
    AsyncTask::new(ValidateTask {
        data: buffer.to_vec(),
        options: options.unwrap_or(ValidationOptions {
            min_score: Some(60.0),
            reject_blurry: Some(false),
            reject_bad_exposure: Some(false),
        }),
    })
}

/// Analyze a batch of images in parallel.
/// Returns a Promise<IqaResult[]> in the same order as input.
#[napi]
pub fn score_batch(buffers: Vec<Buffer>) -> AsyncTask<BatchTask> {
    AsyncTask::new(BatchTask {
        items: buffers.into_iter().map(|b| b.to_vec()).collect(),
    })
}
