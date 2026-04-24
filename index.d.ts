export interface IqaResult {
  /** BRISQUE-derived quality score in [0, 100]. Lower = better quality.
   *  0–20  Excellent | 20–40  Good | 40–60  Acceptable | 60+  Poor */
  score: number;
  /** "excellent" | "good" | "acceptable" | "poor" */
  quality: string;
  /** True when Laplacian-variance blur detection fires */
  isBlurry: boolean;
  /** "normal" | "underexposed" | "overexposed" */
  exposure: string;
  /** "low" | "medium" | "high" */
  noiseLevel: string;
  /** Human-readable issue list. Empty for clean images. */
  warnings: string[];
}

export interface ValidationOptions {
  /** Reject images with score above this value (default 60). */
  minScore?: number;
  /** Reject blurry images (default false). */
  rejectBlurry?: boolean;
  /** Reject non-normal exposures (default false). */
  rejectBadExposure?: boolean;
}

export interface ValidationResult {
  passed: boolean;
  score: number;
  quality: string;
  failures: string[];
}

/** Full quality analysis of a single image. */
export function analyze(imageBuffer: Buffer | Uint8Array): Promise<IqaResult>;

/** Returns only the numeric quality score [0–100]. */
export function quickScore(imageBuffer: Buffer | Uint8Array): Promise<number>;

/** Validate an image against quality thresholds. */
export function validate(
  imageBuffer: Buffer | Uint8Array,
  options?: ValidationOptions
): Promise<ValidationResult>;

/** Analyze a batch of images. Results preserve input order. */
export function scoreBatch(
  imageBuffers: Array<Buffer | Uint8Array>
): Promise<IqaResult[]>;
