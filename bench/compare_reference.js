'use strict';
/**
 * Score the reference image set with native-iqa and compare against
 * OpenCV QualityBRISQUE reference scores produced by compare_opencv.py.
 *
 * Usage:
 *   /path/to/envs/eicon/bin/python3 bench/compare_opencv.py  # generate images + reference CSV
 *   node bench/compare_reference.js                           # compare
 *
 * Exits 0 when all images are within TOLERANCE, 1 otherwise.
 */

const fs = require('fs');
const path = require('path');
const iqa = require('..');

const IMAGES_DIR = path.join(__dirname, 'images');
const REF_CSV    = path.join(__dirname, 'reference_scores.csv');
const TOLERANCE  = 3.0;

function parseCSV(filepath) {
  return fs.readFileSync(filepath, 'utf8')
    .trim()
    .split('\n')
    .slice(1)
    .map(line => {
      const [image, reference_score] = line.split(',');
      return { image: image.trim(), referenceScore: parseFloat(reference_score) };
    });
}

async function main() {
  if (!fs.existsSync(REF_CSV)) {
    console.error('Reference scores not found. Run: python bench/compare_reference.py');
    process.exit(1);
  }

  const refs = parseCSV(REF_CSV);
  const results = [];

  for (const { image, referenceScore } of refs) {
    const imgPath = path.join(IMAGES_DIR, image);
    if (!fs.existsSync(imgPath)) {
      console.warn(`  SKIP ${image} (file not found)`);
      continue;
    }
    const buf = fs.readFileSync(imgPath);
    const nativeScore = await iqa.quickScore(buf);
    const diff = nativeScore - referenceScore;
    results.push({ image, referenceScore, nativeScore, diff });
  }

  const W = 35;
  console.log(`\n${'Image'.padEnd(W)} ${'opencv'.padStart(8)} ${'native'.padStart(8)} ${'diff'.padStart(8)}  status`);
  console.log('-'.repeat(W + 34));

  let pass = 0, fail = 0;
  for (const { image, referenceScore, nativeScore, diff } of results) {
    const ok = Math.abs(diff) <= TOLERANCE;
    ok ? pass++ : fail++;
    const sign = diff >= 0 ? '+' : '';
    const status = ok ? 'OK' : `FAIL (tolerance ±${TOLERANCE})`;
    console.log(
      `${image.padEnd(W)} ${referenceScore.toFixed(2).padStart(8)} ` +
      `${nativeScore.toFixed(2).padStart(8)} ${(sign + diff.toFixed(2)).padStart(8)}  ${status}`
    );
  }

  const diffs = results.map(r => r.diff);
  const mean  = diffs.reduce((a, b) => a + b, 0) / diffs.length;
  const absMax = Math.max(...diffs.map(Math.abs));

  console.log(`\n${pass + fail} images — ${pass} within ±${TOLERANCE}, ${fail} outside`);
  console.log(`Mean offset: ${mean >= 0 ? '+' : ''}${mean.toFixed(2)}   Max abs diff: ${absMax.toFixed(2)}`);


  process.exit(fail > 0 ? 1 : 0);
}

main().catch(err => { console.error(err); process.exit(1); });
