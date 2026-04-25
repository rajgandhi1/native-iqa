'use strict';

/**
 * native-iqa  –  Image Quality Assessment for Node.js
 *
 * Thin JS wrapper around the native Rust/N-API module.
 * All functions accept Buffer or Uint8Array and return Promises.
 */

const { existsSync } = require('fs');
const path = require('path');
const os = require('os');

// ---------------------------------------------------------------------------
// Load native binary
// ---------------------------------------------------------------------------

function loadNative() {
  const platform = os.platform();
  const arch = os.arch();
  const name = `native_iqa.${platform}-${arch}.node`;
  const local = path.join(__dirname, name);

  if (existsSync(local)) {
    return require(local);
  }

  // Fallback: try the unqualified name (useful during development)
  const ext = { darwin: '.dylib', linux: '.so', win32: '.dll' }[platform] ?? '.so';
  const prefix = platform === 'win32' ? '' : 'lib';
  const fallback = path.join(__dirname, 'target', 'release', `${prefix}native_iqa${ext}`);
  if (existsSync(fallback)) {
    return require(fallback);
  }

  throw new Error(
    `native-iqa: could not find compiled native module "${name}".\n` +
    'Run "npm run build" to compile it.'
  );
}

const native = loadNative();

// ---------------------------------------------------------------------------
// Normalise input to Buffer
// ---------------------------------------------------------------------------

function toBuffer(input) {
  if (Buffer.isBuffer(input)) return input;
  if (input instanceof Uint8Array) return Buffer.from(input.buffer, input.byteOffset, input.byteLength);
  throw new TypeError('Expected Buffer or Uint8Array');
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/**
 * Full quality analysis of a single image.
 *
 * @param {Buffer|Uint8Array} imageBuffer  Raw image bytes (JPEG, PNG, WebP, BMP, TIFF)
 * @returns {Promise<{
 *   score:      number,   // BRISQUE-derived quality score [0–100]. Lower = better.
 *   quality:    string,   // "excellent" | "good" | "acceptable" | "poor"
 *   isBlurry:   boolean,
 *   exposure:   string,   // "normal" | "underexposed" | "overexposed"
 *   noiseLevel: string,   // "low" | "medium" | "high"
 *   warnings:   string[]  // human-readable issue list
 * }>}
 */
async function analyze(imageBuffer) {
  const result = await native.analyze(toBuffer(imageBuffer));
  return {
    score:      result.score,
    quality:    result.quality,
    isBlurry:   result.isBlurry,
    exposure:   result.exposure,
    noiseLevel: result.noiseLevel,
    warnings:   result.warnings,
  };
}

/**
 * Returns only the numeric quality score [0–100]. Slightly faster than analyze().
 *
 * @param {Buffer|Uint8Array} imageBuffer
 * @returns {Promise<number>}
 */
async function quickScore(imageBuffer) {
  return native.quickScore(toBuffer(imageBuffer));
}

/**
 * Validate an image against configurable quality thresholds.
 *
 * @param {Buffer|Uint8Array} imageBuffer
 * @param {object} [options]
 * @param {number}  [options.minScore=60]          Reject if score > minScore
 * @param {boolean} [options.rejectBlurry=false]   Reject blurry images
 * @param {boolean} [options.rejectBadExposure=false]
 * @returns {Promise<{ passed: boolean, score: number, quality: string, failures: string[] }>}
 */
async function validate(imageBuffer, options = {}) {
  const opts = {
    minScore:          options.minScore          ?? 60,
    rejectBlurry:      options.rejectBlurry      ?? false,
    rejectBadExposure: options.rejectBadExposure ?? false,
  };
  return native.validate(toBuffer(imageBuffer), opts);
}

/**
 * Analyze a batch of images. Results are returned in the same order as input.
 *
 * @param {Array<Buffer|Uint8Array>} imageBuffers
 * @returns {Promise<Array<ReturnType<analyze>>>}
 */
async function scoreBatch(imageBuffers) {
  if (!Array.isArray(imageBuffers)) {
    throw new TypeError('scoreBatch: expected an array of buffers');
  }
  const buffers = imageBuffers.map(toBuffer);
  const results = await native.scoreBatch(buffers);
  return results.map((r) => ({
    score:      r.score,
    quality:    r.quality,
    isBlurry:   r.isBlurry,
    exposure:   r.exposure,
    noiseLevel: r.noiseLevel,
    warnings:   r.warnings,
  }));
}

module.exports = { analyze, quickScore, validate, scoreBatch };
