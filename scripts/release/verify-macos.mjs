import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync, spawn } from 'node:child_process';
import { setTimeout as delay } from 'node:timers/promises';

assert.equal(process.platform, 'darwin', 'Run macOS package verification on a Mac');
const target = process.env.RELEASE_TARGET;
const expectedArch = { 'aarch64-apple-darwin': 'arm64', 'x86_64-apple-darwin': 'x86_64' }[target];
assert.ok(expectedArch, 'Set a native macOS RELEASE_TARGET');
const notarized = process.env.REQUIRE_NOTARIZATION === 'true';
const config = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8'));
const root = path.resolve(`src-tauri/target/${target}/release/bundle`);
const evidence = path.resolve('qa-output/release');
fs.mkdirSync(evidence, { recursive: true });
const report = { target, notarizationRequired: notarized, checks: [], launch: null };
const run = (program, args) => {
  const value = execFileSync(program, args, { encoding: 'utf8', timeout: 120_000 });
  report.checks.push({ program, args, passed: true });
  return value;
};
const verifyApp = app => {
  const plist = path.join(app, 'Contents/Info.plist');
  const executable = run('/usr/libexec/PlistBuddy', ['-c', 'Print :CFBundleExecutable', plist]).trim();
  assert.equal(run('/usr/libexec/PlistBuddy', ['-c', 'Print :CFBundleShortVersionString', plist]).trim(), config.version);
  const binary = path.join(app, 'Contents/MacOS', executable);
  assert.deepEqual(run('lipo', ['-archs', binary]).trim().split(/\s+/), [expectedArch]);
  run('codesign', ['--verify', '--deep', '--strict', '--verbose=2', app]);
  if (notarized) {
    // Gatekeeper checks the Developer ID chain and accepted notarization ticket.
    run('xcrun', ['stapler', 'validate', app]);
    run('spctl', ['--assess', '--type', 'execute', '--verbose=4', app]);
    const requirement = `anchor apple generic and certificate leaf[subject.OU] = "${process.env.APPLE_TEAM_ID}" and certificate leaf[field.1.2.840.113635.100.6.1.13] exists`;
    assert.match(process.env.APPLE_TEAM_ID || '', /^[A-Z0-9]{10}$/);
    run('codesign', ['--verify', '--strict', '-R', requirement, app]);
  }
  return executable;
};
const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'file2file-package-'));
const mount = path.join(temporary, 'mounted');
fs.mkdirSync(mount);
let attached = false;
let child;
try {
  const dmgs = fs.readdirSync(path.join(root, 'dmg')).filter(name => name.endsWith('.dmg'));
  assert.equal(dmgs.length, 1, 'Expected exactly one architecture-specific disk image');
  const dmg = path.join(root, 'dmg', dmgs[0]);
  run('hdiutil', ['verify', dmg]);
  run('hdiutil', ['attach', '-readonly', '-nobrowse', '-mountpoint', mount, dmg]);
  attached = true;
  const app = path.join(mount, `${config.productName}.app`);
  verifyApp(app);
  // Test a copied installation, not just the staging bundle used by the packager.
  const installed = path.join(temporary, `${config.productName}.app`);
  run('ditto', [app, installed]);
  const executable = verifyApp(installed);
  run('hdiutil', ['detach', mount]);
  attached = false;
  const windowChecker = path.join(temporary, 'check-window');
  run('swiftc', ['scripts/release/check-macos-window.swift', '-o', windowChecker]);
  const log = fs.openSync(path.join(evidence, 'launch.log'), 'w');
  child = spawn(path.join(installed, 'Contents/MacOS', executable), [], { stdio: ['ignore', log, log] });
  fs.closeSync(log);
  const exited = new Promise(resolve => {
    child.once('error', error => resolve({ error: error.message }));
    child.once('exit', (code, signal) => resolve({ code, signal }));
  });
  const result = await Promise.race([exited, delay(8000).then(() => ({ running: true }))]);
  report.launch = result;
  assert.equal(result.running, true, 'The installed application exited during launch; inspect launch.log');
  const deadline = Date.now() + 60_000;
  do {
    report.window = JSON.parse(execFileSync(windowChecker, [String(child.pid), path.join(evidence, 'window.png')], { encoding: 'utf8', timeout: 15_000 }));
    if (report.window.nonBlank) break;
    assert.equal(child.exitCode, null, 'Application exited before its window rendered');
    await delay(2000);
  } while (Date.now() < deadline);
  assert.equal(report.window?.nonBlank, true, 'App window stayed blank; inspect window.png and launch.log');
  try {
    run('screencapture', ['-x', path.join(evidence, 'desktop.png')]);
  } catch {
    report.screenshot = 'Unavailable in this runner session; process launch was checked.';
  }
  child.kill('SIGTERM');
  await Promise.race([exited, delay(3000)]);
  if (child.exitCode === null && child.signalCode === null) child.kill('SIGKILL');
  report.passed = true;
  console.log(`Verified ${expectedArch} DMG, installed app signature${notarized ? ', notarization and Gatekeeper' : ' (ad-hoc preview only)'}, and process launch`);
} catch (error) {
  report.passed = false;
  report.error = error.message;
  throw error;
} finally {
  if (child?.exitCode === null && child?.signalCode === null) child.kill('SIGKILL');
  if (attached) {
    try { execFileSync('hdiutil', ['detach', mount], { timeout: 30_000 }); attached = false; }
    catch (error) { report.cleanupError = error.message; }
  }
  fs.writeFileSync(path.join(evidence, 'verification.json'), JSON.stringify(report, null, 2));
  if (!attached) fs.rmSync(temporary, { recursive: true, force: true });
}
