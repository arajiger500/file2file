import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
const json = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const version = json('package.json').version;
assert.match(version, /^[0-9]+[.][0-9]+[.][0-9]+$/);
assert.equal(json('package-lock.json').version, version);
assert.equal(json('package-lock.json').packages[''].version, version);
const config = json('src-tauri/tauri.conf.json');
assert.equal(config.version, version);
assert.equal(config.bundle.externalBin?.length || 0, 0, 'Unverified sidecar packaging is prohibited');
assert.equal(fs.readFileSync('src-tauri/Cargo.toml', 'utf8').match(/\[package\][\s\S]*?\nversion = "([^"]+)"/)[1], version);
assert.equal(fs.readFileSync('src-tauri/Cargo.lock', 'utf8').match(/\[\[package\]\]\nname = "file2file"\nversion = "([^"]+)"/)[1], version);
for (const icon of config.bundle.icon) assert.ok(fs.statSync(path.join('src-tauri', icon)).size > 0, icon);
if (process.env.GITHUB_REF_TYPE === 'tag') assert.equal(process.env.GITHUB_REF_NAME, `v${version}`, 'Tag must match every version manifest');
const target = process.env.RELEASE_TARGET;
if (target) assert.equal(execFileSync('rustc', ['-vV'], { encoding: 'utf8' }).match(/^host: (.+)$/m)?.[1], target, 'Build must use the native architecture');
if (process.argv.includes('--artifacts')) {
  assert.ok(target, 'RELEASE_TARGET is required');
  const root = `src-tauri/target/${target}/release/bundle`;
  const files = fs.readdirSync(root, { recursive: true }).map(name => path.join(root, name)).filter(file => fs.statSync(file).isFile());
  const extensions = target.includes('windows') ? ['.msi', '.exe'] : target.includes('apple') ? ['.dmg'] : ['.deb', '.AppImage'];
  for (const extension of extensions) {
    const packages = files.filter(file => file.endsWith(extension));
    assert.ok(packages.length > 0, `Missing ${extension} package`);
    for (const file of packages) assert.ok(fs.statSync(file).size > 100_000, `Suspiciously small package: ${file}`);
  }
}
console.log(`Verified File2File ${version}${target ? ` (${target})` : ''}`);
