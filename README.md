# native-iqa

No-reference image quality assessment for Node.js. Runs BRISQUE feature extraction plus blur, exposure, and noise detection in Rust via N-API. All processing happens off the JS thread.

Accepts JPEG, PNG, WebP, BMP, and TIFF. Returns scores in under 10ms for typical web images.

```
npm install native-iqa
```

Requires Node.js 18 or later. No Rust toolchain needed.

---

## Usage

```js
const iqa = require('native-iqa');
const fs = require('fs');

const buffer = fs.readFileSync('photo.jpg');
const result = await iqa.analyze(buffer);
// {
//   score: 28.4,
//   quality: 'good',
//   isBlurry: false,
//   exposure: 'normal',
//   noiseLevel: 'low',
//   warnings: []
// }
```

---

## API

### analyze(buffer)

Full quality analysis. Returns a Promise.

```js
const result = await iqa.analyze(buffer);
```

```
result.score        number    Quality score 0-100. Lower is better.
result.quality      string    'excellent' | 'good' | 'acceptable' | 'poor'
result.isBlurry     boolean   True when Laplacian variance is below threshold
result.exposure     string    'normal' | 'underexposed' | 'overexposed'
result.noiseLevel   string    'low' | 'medium' | 'high'
result.warnings     string[]  Human-readable list of detected issues
```

Score bands:

```
0-20    excellent
20-40   good
40-60   acceptable
60+     poor
```

The score is computed by a pre-trained SVR model using 36 BRISQUE features extracted from MSCN coefficients across two scales. Natural, sharp photos score low. Flat, blurry, or noisy images score high.

---

### quickScore(buffer)

Returns only the numeric score. Faster than `analyze` because it skips blur, exposure, and noise detection entirely.

```js
const score = await iqa.quickScore(buffer);
// 28.4
```

---

### validate(buffer, options)

Returns a pass/fail result against configurable thresholds.

```js
const result = await iqa.validate(buffer, {
  maxScore: 50,
  rejectBlurry: true,
  rejectBadExposure: true,
});
// { passed: true, score: 28.4, quality: 'good', failures: [] }
```

Options:

```
maxScore           number    Reject if score exceeds this value. Default 60.
rejectBlurry       boolean   Reject blurry images. Default false.
rejectBadExposure  boolean   Reject underexposed or overexposed images. Default false.
```

---

### scoreBatch(buffers)

Analyze multiple images in parallel. Uses a Rayon thread pool internally so throughput scales with available cores. Results are returned in the same order as input.

```js
const results = await iqa.scoreBatch([bufferA, bufferB, bufferC]);
```

Each item in the result array has the same shape as analyze().

---

## Supported platforms

Prebuilt binaries are provided for:

- macOS arm64 (Apple Silicon)
- macOS x64
- Linux x64 (glibc)
- Linux arm64 (glibc)
- Windows x64

---

## Building from source

Requires Rust 1.88 or later and the napi-rs CLI.

```
npm run build
npm test
```

---

## Benchmark

```
npm run bench
```

Runs `analyze`, `quickScore`, and `scoreBatch` at 256, 512, and 1024px over 50 iterations and prints median and p95 latency. No extra dependencies required.

---

## Accuracy

BRISQUE scores match OpenCV's `QualityBRISQUE` (LIVE model) within ±1 point across a representative test set covering sharp, blurry, noisy, dark, and bright images. The small residual (~0.5 mean offset) is due to float64 vs float32 precision.

---

## License

MIT
