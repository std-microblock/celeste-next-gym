#!/usr/bin/env node
// cli.mjs -- flatten a Celeste .tas file into a canonical per-frame input stream.
//
//   node src/cli.mjs <tasRoot> <entryFile> [--out out.json] [--summary]
//
// Extra (non-required) flags:
//   --table        print the <entryFile> vs its own FileTime header, plus every
//                  other .tas file in <tasRoot> that carries a header
//   --events       include the event list in the --summary output
//   --json         print the full JSON on stdout even when --summary is given
//   --quiet        suppress the summary
//
// ASCII only.

import fs from 'node:fs';
import path from 'node:path';
import { resolveTas, parseFileTimeHeader } from './resolve.mjs';

function usage(message) {
  if (message) process.stderr.write(`error: ${message}\n\n`);
  process.stderr.write(
    'usage: node src/cli.mjs <tasRoot> <entryFile> [--out out.json] [--summary]\n'
    + '       extra flags: --table --events --json --quiet\n',
  );
  process.exit(2);
}

const argv = process.argv.slice(2);
const positional = [];
const flags = new Set();
let outPath = null;

for (let i = 0; i < argv.length; i++) {
  const a = argv[i];
  if (a === '--out') {
    if (i + 1 >= argv.length) usage('--out needs a path');
    outPath = argv[++i];
  } else if (a === '--summary') {
    flags.add('summary');
  } else if (a === '--table') {
    flags.add('table');
  } else if (a === '--events') {
    flags.add('events');
  } else if (a === '--json') {
    flags.add('json');
  } else if (a === '--quiet') {
    flags.add('quiet');
  } else if (a.startsWith('--')) {
    usage(`unknown flag ${a}`);
  } else {
    positional.push(a);
  }
}

if (positional.length !== 2) usage('expected <tasRoot> and <entryFile>');
const [tasRoot, entryFile] = positional;

if (!fs.existsSync(tasRoot)) usage(`no such directory: ${tasRoot}`);
const entryAbs = path.resolve(tasRoot, entryFile);
if (!fs.existsSync(entryAbs)) usage(`no such TAS file: ${entryAbs}`);

const warnings = [];
const resolution = resolveTas(tasRoot, entryFile, {
  onWarning: (m) => warnings.push(m),
});

function fmtFramesAsHeaderTime(frames) {
  // The header writes TimeSpan.FromTicks(N * 170000); see docs/tas-format.md.
  const ms = frames * 17;
  const h = Math.floor(ms / 3600000);
  const m = Math.floor((ms % 3600000) / 60000);
  const s = Math.floor((ms % 60000) / 1000);
  const milli = ms % 1000;
  const tail = `${String(s).padStart(2, '0')}.${String(milli).padStart(3, '0')}`;
  return h > 0 ? `${h}:${String(m).padStart(2, '0')}:${tail}` : `${m}:${tail}`;
}

function headerInfo(absPath) {
  const text = fs.readFileSync(absPath, 'utf8');
  return parseFileTimeHeader(text);
}

function printSummary() {
  const h = headerInfo(entryAbs);
  const lines = [];
  lines.push(`entry              : ${entryFile}`);
  lines.push(`totalFrames        : ${resolution.totalFrames}`);
  lines.push(`input entries      : ${resolution.inputs.length}`);
  lines.push(`events             : ${resolution.events.length}`);
  if (h) {
    lines.push(`FileTime header    : ${h.raw}  ->  N = ${h.frames}  (= ${fmtFramesAsHeaderTime(h.frames)})`);
    lines.push(`  parsed - N       : ${resolution.totalFrames - h.frames} frames that the save-file timer did not count`);
    lines.push('  (N is (SaveData.Time - start)/Engine.RawDeltaTime.SecondsToTicks(), not');
    lines.push('   InputController.Inputs.Count; see docs/tas-format.md "The FileTime header")');
  } else {
    lines.push('FileTime header    : (none)');
  }

  const byType = new Map();
  for (const ev of resolution.events) byType.set(ev.type, (byType.get(ev.type) ?? 0) + 1);
  lines.push('directives         : ' + [...byType.entries()]
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
    .map(([t, n]) => `${t}=${n}`).join(' '));

  const byFile = new Map();
  for (const inp of resolution.inputs) byFile.set(inp.file, (byFile.get(inp.file) ?? 0) + inp.frames);
  lines.push(`files used         : ${byFile.size}`);
  if (flags.has('events')) {
    lines.push('--- events ---');
    for (const ev of resolution.events) {
      lines.push(`  ${String(ev.frame).padStart(7)}  ${ev.type}${ev.args.length ? ' ' + ev.args.join(' | ') : ''}  (${ev.file}:${ev.line})`);
    }
  }
  if (warnings.length > 0) {
    lines.push('warnings           :');
    for (const w of warnings) lines.push(`  - ${w}`);
  }
  process.stdout.write(lines.join('\n') + '\n');
}

function walkTas(dir, out = []) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) walkTas(p, out);
    else if (e.name.endsWith('.tas')) out.push(p);
  }
  return out;
}

function printTable() {
  const rows = [];
  for (const abs of walkTas(path.resolve(tasRoot))) {
    const h = headerInfo(abs);
    if (!h) continue;
    const rel = path.relative(path.resolve(tasRoot), abs).split(path.sep).join('/');
    let computed = null;
    let error = null;
    try {
      const warn = [];
      computed = resolveTas(tasRoot, rel, { onWarning: (m) => warn.push(m) }).totalFrames;
    } catch (e) {
      error = e.message;
    }
    rows.push({ rel, computed, header: h.frames, error });
  }
  rows.sort((a, b) => a.rel.localeCompare(b.rel));
  const w = Math.max(24, ...rows.map((r) => r.rel.length));
  process.stdout.write(`${'file'.padEnd(w)}  ${'computed'.padStart(9)}  ${'header'.padStart(9)}  ${'computed-header'.padStart(15)}\n`);
  process.stdout.write(`${'-'.repeat(w)}  ${'-'.repeat(9)}  ${'-'.repeat(9)}  ${'-'.repeat(15)}\n`);
  for (const r of rows) {
    if (r.error) {
      process.stdout.write(`${r.rel.padEnd(w)}  ${'ERROR'.padStart(9)}  ${String(r.header).padStart(9)}  ${r.error}\n`);
    } else {
      process.stdout.write(`${r.rel.padEnd(w)}  ${String(r.computed).padStart(9)}  ${String(r.header).padStart(9)}  ${String(r.computed - r.header).padStart(15)}\n`);
    }
  }
  process.stdout.write(`\n${rows.length} file(s) with a FileTime header\n`);
}

if (outPath) {
  const out = {
    version: resolution.version,
    entry: resolution.entry,
    totalFrames: resolution.totalFrames,
    inputs: resolution.inputs,
    events: resolution.events,
  };
  fs.mkdirSync(path.dirname(path.resolve(outPath)), { recursive: true });
  fs.writeFileSync(path.resolve(outPath), JSON.stringify(out, null, 2) + '\n');
  process.stdout.write(`wrote ${outPath} (${resolution.totalFrames} frames, ${resolution.inputs.length} input entries, ${resolution.events.length} events)\n`);
}

if (flags.has('table')) printTable();
if (flags.has('json') && !outPath) {
  process.stdout.write(JSON.stringify(resolution, null, 2) + '\n');
} else if (!flags.has('quiet')) {
  printSummary();
}
