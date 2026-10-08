#!/usr/bin/env node
// Applies the TasFrameTrace instrumented build to a CelesteTAS-EverestInterop checkout.
//
//   node tools/celestetas-trace/apply.mjs <celestetas-src-dir> [--no-build]
//
// The patch is idempotent: running it twice produces the same tree.
// It never modifies the source checkout you pass in? No - it DOES modify it, so pass a copy.

import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

const [, , srcDir, ...rest] = process.argv;
const noBuild = rest.includes('--no-build');

if (!srcDir) {
  console.error('usage: node apply.mjs <celestetas-src-dir> [--no-build]');
  process.exit(2);
}

const root = path.resolve(srcDir);
const toolsDir = path.join(root, 'CelesteTAS-EverestInterop', 'Source', 'Tools');
const controllerPath = path.join(root, 'CelesteTAS-EverestInterop', 'Source', 'TAS', 'Input', 'InputController.cs');
const csproj = path.join(root, 'CelesteTAS-EverestInterop', 'CelesteTAS-EverestInterop.csproj');

for (const p of [toolsDir, controllerPath, csproj]) {
  if (!fs.existsSync(p)) {
    console.error(`not a CelesteTAS source checkout, missing: ${p}`);
    process.exit(2);
  }
}

// 1. drop the exporter into the project (SDK-style projects glob **/*.cs).
const target = path.join(toolsDir, 'TasFrameTrace.cs');
fs.copyFileSync(path.join(import.meta.dirname, 'TasFrameTrace.cs'), target);
console.log(`wrote ${target}`);

// 2. hook it into the one place CelesteTAS advances a TAS frame, next to its own exporter. This call
//    site is deliberate: `CurrentFrameInTas` is still the frame about to run, and AdvanceFrame always
//    ends with exactly one `CurrentFrameInTas++`.
let controller = fs.readFileSync(controllerPath, 'utf8');
// Tolerate LF and CRLF checkouts.
const anchor = /^([ \t]*)ExportGameInfo\.ExportInfo\(\);\r?\n/m;
// The press edges must be sampled after `InputHelper.FeedInputs` (which ends with
// `MInput.UpdateVirtualInputs`) and before the frame's `Scene.Update`, because `Player.BoostUpdate`
// calls `Input.Dash.ConsumePress()` and `VirtualButton.Pressed` is false afterwards.
const feedAnchor = /^([ \t]*)InputHelper\.FeedInputs\(Current!\);\r?\n/m;

let patched = false;
if (controller.includes('TasFrameTrace.ExportInfo();')) {
  console.log('InputController.cs already patched (ExportInfo)');
} else if (anchor.test(controller)) {
  controller = controller.replace(anchor, (match, indent) => `${match}${indent}TasFrameTrace.ExportInfo();\n`);
  console.log('patched InputController.cs (ExportInfo)');
  patched = true;
} else {
  console.error('could not find the "ExportGameInfo.ExportInfo();" anchor in InputController.cs; patch manually');
  process.exit(1);
}

if (controller.includes('TasFrameTrace.CaptureInput();')) {
  console.log('InputController.cs already patched (CaptureInput)');
} else if (feedAnchor.test(controller)) {
  controller = controller.replace(feedAnchor, (match, indent) => `${match}${indent}TasFrameTrace.CaptureInput();\n`);
  console.log('patched InputController.cs (CaptureInput)');
  patched = true;
} else {
  console.error('could not find the "InputHelper.FeedInputs(Current!);" anchor in InputController.cs; patch manually');
  process.exit(1);
}

if (patched) {
  fs.writeFileSync(controllerPath, controller);
  console.log(`wrote ${controllerPath}`);
}

if (noBuild) {
  process.exit(0);
}

const build = spawnSync(
  process.platform === 'win32' ? 'dotnet.exe' : 'dotnet',
  ['build', csproj, '-c', 'Release', '-p:UseSymlinks=false'],
  { stdio: 'inherit' },
);
process.exit(build.status ?? 1);
