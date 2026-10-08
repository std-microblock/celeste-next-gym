// Streaming readers for TAS fidelity artifacts.
//
// WHY THIS EXISTS: a scratch script once did
//     JSON.parse(fs.readFileSync('.../some.json', 'utf8'))
// on a multi-gigabyte file. V8 turned the string plus the object graph into 66-74 GB of private
// memory, the box ran out of physical RAM, and the hard-fault storm froze input for minutes.
//
// RULE, no exceptions: never `readFileSync` + `JSON.parse` an artifact whose size you have not
// checked. Use `readJsonSmall` (which refuses to exceed a cap) or a streaming reader below.
// The canonical ground-truth traces (`trace-*.jsonl`, 0.9-1.7 GB) are line-delimited on purpose:
// every consumer must stream them, one line at a time.

import fs from 'node:fs';
import readline from 'node:readline';

/** Default ceiling for a whole-file JSON read. Anything larger must be streamed. */
export const SMALL_JSON_LIMIT_BYTES = 64 * 1024 * 1024;

export class ArtifactTooLarge extends Error {
  constructor(path, bytes, limit) {
    super(
      `refusing to load ${path} (${(bytes / 1048576).toFixed(1)} MB) into memory; ` +
        `the cap is ${(limit / 1048576).toFixed(0)} MB. Stream it instead ` +
        `(forEachJsonLine / forEachJsonArrayItem), or rewrite the artifact as NDJSON.`,
    );
    this.name = 'ArtifactTooLarge';
    this.path = path;
    this.bytes = bytes;
    this.limit = limit;
  }
}

/** `JSON.parse(readFileSync(...))`, but it throws instead of exhausting memory. */
export function readJsonSmall(path, limit = SMALL_JSON_LIMIT_BYTES) {
  const { size } = fs.statSync(path);
  if (size > limit) {
    throw new ArtifactTooLarge(path, size, limit);
  }
  return JSON.parse(fs.readFileSync(path, 'utf8'));
}

/** Stream a `.jsonl`/`.ndjson` file one parsed record at a time. */
export async function forEachJsonLine(path, visit) {
  const rl = readline.createInterface({ input: fs.createReadStream(path), crlfDelay: Infinity });
  let index = 0;
  for await (const line of rl) {
    if (!line) continue;
    await visit(JSON.parse(line), index++);
  }
  return index;
}

/**
 * Stream the **elements** of a top-level JSON array without materialising the array.
 *
 * Handles the shape the fidelity gate writes (`{"segments":[...], "totals":{...}}`) and a bare
 * `[...]`. It is a scanner, not a parser: it finds the target array and then tracks brace and
 * string state so it can hand complete elements to `visit` one at a time.
 */
export async function forEachJsonArrayItem(path, key, visit) {
  const stream = fs.createReadStream(path, { encoding: 'utf8', highWaterMark: 1 << 20 });
  const needle = key === null ? null : `"${key}"`;

  let buffer = '';
  let started = false; // have we found the opening '[' of the target array?
  let depth = 0;
  let inString = false;
  let escaped = false;
  let current = '';
  let index = 0;
  let primedHeader = false;

  for await (const chunk of stream) {
    buffer += chunk;

    if (!started) {
      let at = -1;
      if (!primedHeader || needle === null) {
        at = needle === null ? 0 : buffer.indexOf(needle);
        if (needle !== null && at === -1) {
          // keep only a tail that could still hold a partial key
          if (buffer.length > 4096) buffer = buffer.slice(-4096);
          continue;
        }
        primedHeader = true;
        buffer = needle === null ? buffer : buffer.slice(at + needle.length);
      } else {
        primedHeader = true;
      }
      const open = buffer.indexOf('[');
      if (open === -1) {
        if (buffer.length > 4096) buffer = buffer.slice(-4096);
        continue;
      }
      buffer = buffer.slice(open + 1);
      started = true;
    }

    for (let i = 0; i < buffer.length; i++) {
      const c = buffer[i];
      if (escaped) {
        if (depth > 0 || current.length > 0) current += c;
        escaped = false;
        continue;
      }
      if (inString) {
        if (c === '\\') escaped = true;
        if (depth > 0 || current.length > 0) current += c;
        if (c === '"') inString = false;
        continue;
      }
      if (c === '"') {
        inString = true;
        current += c;
        continue;
      }
      if (c === '[' || c === '{') {
        depth++;
        current += c;
        continue;
      }
      if (c === ']' || c === '}') {
        if (depth === 0 && c === ']') {
          if (current.trim()) await visit(JSON.parse(current), index++);
          return index;
        }
        depth--;
        current += c;
        if (depth === 0) {
          await visit(JSON.parse(current), index++);
          current = '';
        }
        continue;
      }
      if (c === ',' && depth === 0) {
        if (current.trim()) await visit(JSON.parse(current), index++);
        current = '';
        continue;
      }
      if (depth > 0 || current.length > 0) current += c;
    }
    buffer = '';
  }
  if (current.trim()) await visit(JSON.parse(current), index++);
  return index;
}

/** Small helper so callers can report honestly without loading anything. */
export function describeArtifact(path) {
  const { size } = fs.statSync(path);
  return { path, bytes: size, mb: +(size / 1048576).toFixed(1) };
}
