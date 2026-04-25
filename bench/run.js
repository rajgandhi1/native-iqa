'use strict';

/**
 * Benchmark for native-iqa: analyze, quickScore, scoreBatch
 *
 * Generates synthetic PNG images at 256, 512, and 1024px, runs 50 iterations
 * of each API, and reports median + p95 latency. No external dependencies.
 *
 * Usage: node bench/run.js
 */

const zlib = require('zlib');
const iqa = require('..');

// ---------------------------------------------------------------------------
// Minimal PNG generator (grayscale, no external deps)
// ---------------------------------------------------------------------------

function makePng(width, height, getPixel) {
  const rowSize = width;
  const raw = Buffer.allocUnsafe(height * (1 + rowSize));
  for (let y = 0; y < height; y++) {
    raw[y * (1 + rowSize)] = 0;
    for (let x = 0; x < width; x++) {
      raw[y * (1 + rowSize) + 1 + x] = getPixel(x, y);
    }
  }

  const compressed = zlib.deflateSync(raw);

  function crc32(buf) {
    const table = (() => {
      const t = new Uint32Array(256);
      for (let n = 0; n < 256; n++) {
        let c = n;
        for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
        t[n] = c;
      }
      return t;
    })();
    let c = 0xffffffff;
    for (const byte of buf) c = table[(c ^ byte) & 0xff] ^ (c >>> 8);
    return (c ^ 0xffffffff) >>> 0;
  }

  function chunk(type, data) {
    const typeBytes = Buffer.from(type, 'ascii');
    const len = Buffer.allocUnsafe(4);
    len.writeUInt32BE(data.length, 0);
    const crcBuf = Buffer.concat([typeBytes, data]);
    const crcVal = Buffer.allocUnsafe(4);
    crcVal.writeUInt32BE(crc32(crcBuf), 0);
    return Buffer.concat([len, typeBytes, data, crcVal]);
  }

  const sig = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const ihdr = Buffer.allocUnsafe(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8;   // bit depth
  ihdr[9] = 0;   // grayscale
  ihdr[10] = 0; ihdr[11] = 0; ihdr[12] = 0;

  return Buffer.concat([sig, chunk('IHDR', ihdr), chunk('IDAT', compressed), chunk('IEND', Buffer.alloc(0))]);
}

// ---------------------------------------------------------------------------
// Stats helpers
// ---------------------------------------------------------------------------

function median(sorted) {
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 0 ? (sorted[mid - 1] + sorted[mid]) / 2 : sorted[mid];
}

function p95(sorted) {
  return sorted[Math.ceil(sorted.length * 0.95) - 1];
}

// ---------------------------------------------------------------------------
// Benchmark runner
// ---------------------------------------------------------------------------

async function bench(label, fn, iterations) {
  // warmup
  for (let i = 0; i < 3; i++) await fn();

  const times = [];
  for (let i = 0; i < iterations; i++) {
    const t0 = performance.now();
    await fn();
    times.push(performance.now() - t0);
  }
  times.sort((a, b) => a - b);
  console.log(`  ${label.padEnd(40)} median=${median(times).toFixed(2)}ms  p95=${p95(times).toFixed(2)}ms`);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

const ITERATIONS = 50;
const SIZES = [256, 512, 1024];

(async () => {
  console.log(`native-iqa benchmark — ${ITERATIONS} iterations per case\n`);

  for (const size of SIZES) {
    const buf = makePng(size, size, (x, y) => ((x * 7 + y * 3) ^ (x ^ y)) & 0xff);
    console.log(`[${size}x${size}]`);
    await bench('analyze()', () => iqa.analyze(buf), ITERATIONS);
    await bench('quickScore()', () => iqa.quickScore(buf), ITERATIONS);
    await bench('scoreBatch([img])', () => iqa.scoreBatch([buf]), ITERATIONS);
    await bench('scoreBatch([img x5])', () => iqa.scoreBatch([buf, buf, buf, buf, buf]), ITERATIONS);
    console.log('');
  }
})();
