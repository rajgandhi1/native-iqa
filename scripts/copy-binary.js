#!/usr/bin/env node
/**
 * Post-build script: copies the compiled cdylib to a platform-specific .node file.
 *
 * Naming convention used by @napi-rs tooling:
 *   <binaryName>.<platform>-<arch>.node
 */

const fs = require('fs');
const path = require('path');
const os = require('os');

const ROOT = path.resolve(__dirname, '..');
const RELEASE = path.join(ROOT, 'target', 'release');

const platform = os.platform(); // 'darwin' | 'linux' | 'win32'
const arch = os.arch();         // 'arm64' | 'x64'

const srcMap = {
  darwin: 'libnative_iqa.dylib',
  linux:  'libnative_iqa.so',
  win32:  'native_iqa.dll',
};

const src = path.join(RELEASE, srcMap[platform]);
const dst = path.join(ROOT, `native_iqa.${platform}-${arch}.node`);

if (!fs.existsSync(src)) {
  console.error(`Build artifact not found: ${src}`);
  process.exit(1);
}

fs.copyFileSync(src, dst);
console.log(`Copied ${path.basename(src)} → ${path.basename(dst)}`);
