#!/usr/bin/env node
// `npm ci` on some Linux setups silently skips sharp's prebuilt @img
// optional packages (the platform packages that carry the native binary).
// sharp then falls back to a source build, which fails, and npm rolls the
// whole install back. Verify that sharp actually loads; if it does not,
// install the @img packages pinned in package-lock.json explicitly.
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const sharpLoads = () => {
  try {
    execFileSync(process.execPath, ['-e', "import('sharp').then(() => process.exit(0), () => process.exit(1))"], { cwd: root, stdio: 'ignore' });
    return true;
  } catch { return false; }
};

if (sharpLoads()) {
  console.log('sharp prebuilt binaries OK.');
  process.exit(0);
}
const lock = JSON.parse(readFileSync(join(root, 'package-lock.json'), 'utf8'));
const libc = process.report?.getReport()?.header?.glibcVersionRuntime ? 'glibc' : 'musl';
const fits = (info, key, value) => !info[key] || info[key].includes(value);
const wanted = [];
for (const [path, info] of Object.entries(lock.packages ?? {})) {
  const match = path.match(/node_modules\/(@img\/[^/]+)$/);
  if (!match || !info.version) continue;
  if (!fits(info, 'os', process.platform) || !fits(info, 'cpu', process.arch) || !fits(info, 'libc', libc)) continue;
  wanted.push(`${match[1]}@${info.version}`);
}
if (!wanted.length) {
  console.error('sharp does not load and package-lock.json pins no @img packages to repair it with.');
  process.exit(1);
}
console.log('sharp does not load; installing its prebuilt packages explicitly:', wanted.join(' '));
execFileSync('npm', ['install', '--no-save', '--ignore-scripts', '--no-audit', '--no-fund', ...wanted], { cwd: root, stdio: 'inherit' });
if (!sharpLoads()) {
  console.error('sharp still does not load after installing the @img packages.');
  process.exit(1);
}
console.log('sharp prebuilt binaries repaired.');
