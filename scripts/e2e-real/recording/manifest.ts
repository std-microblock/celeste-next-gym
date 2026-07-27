import { createHash } from "node:crypto";
import { readFile, realpath, stat } from "node:fs/promises";
import path from "node:path";

export const RAW_WIDTH = 320;
export const RAW_HEIGHT = 180;
export const RAW_PIXEL_FORMAT = "bgra";
export const RAW_FRAME_BYTES = RAW_WIDTH * RAW_HEIGHT * 4;

export interface StateIndexRange {
  start_state_index: number;
  end_state_index: number;
}

export interface PresentationFrame {
  render_index: number;
  state_index: number;
  timestamp_ns: number;
  path: string;
  sha256: string;
  bytes: number;
  repeated_state_presentation: boolean;
  unpresented_updates_before?: StateIndexRange;
}

export interface PresentationManifest {
  schema_version: 1;
  capture_semantics: "presentation_frames";
  scenario_id: string;
  run_nonce: string;
  process_id: number;
  capture_token_sha256: string;
  width: 320;
  height: 180;
  pixel_format: "bgra";
  encoding_frame_rate: 60;
  started_at: string;
  finalized_at: string;
  outcome: string;
  reason?: string;
  start_state_index: number;
  end_state_index: number;
  latest_state_index: number;
  final_state_presented: boolean;
  repeated_presentation_count: number;
  unpresented_update_ranges: StateIndexRange[];
  frames: PresentationFrame[];
}

export interface LoadedPresentationManifest {
  manifest: PresentationManifest;
  manifestPath: string;
  sessionDirectory: string;
  framePaths: string[];
}

export async function loadPresentationManifest(
  recordingRoot: string,
  manifestPath: string,
): Promise<LoadedPresentationManifest> {
  const canonicalRoot = await canonicalDirectory(recordingRoot);
  const canonicalManifest = await realpath(manifestPath);
  ensurePathInside(canonicalRoot, canonicalManifest, "manifest");
  const parsed = JSON.parse(await readFile(canonicalManifest, "utf8")) as unknown;
  const manifest = validateManifest(parsed);
  const sessionDirectory = path.dirname(canonicalManifest);
  const framePaths: string[] = [];

  for (const frame of manifest.frames) {
    const candidate = path.resolve(sessionDirectory, frame.path);
    ensurePathInside(sessionDirectory, candidate, `frame ${frame.render_index}`);
    const canonicalFrame = await realpath(candidate);
    ensurePathInside(canonicalRoot, canonicalFrame, `frame ${frame.render_index}`);
    ensurePathInside(sessionDirectory, canonicalFrame, `frame ${frame.render_index}`);
    const info = await stat(canonicalFrame);
    if (!info.isFile() || info.size !== RAW_FRAME_BYTES || frame.bytes !== RAW_FRAME_BYTES) {
      throw new Error(`frame ${frame.render_index} is not a ${RAW_FRAME_BYTES}-byte BGRA file`);
    }
    const bytes = await readFile(canonicalFrame);
    const actualHash = createHash("sha256").update(bytes).digest("hex");
    if (actualHash !== frame.sha256) {
      throw new Error(`frame ${frame.render_index} SHA-256 mismatch`);
    }
    framePaths.push(canonicalFrame);
  }
  return {
    manifest,
    manifestPath: canonicalManifest,
    sessionDirectory,
    framePaths,
  };
}

export function validateManifest(value: unknown): PresentationManifest {
  const root = requireRecord(value, "manifest");
  if (root.schema_version !== 1 || root.capture_semantics !== "presentation_frames") {
    throw new Error("unsupported presentation manifest schema");
  }
  if (root.width !== RAW_WIDTH || root.height !== RAW_HEIGHT || root.pixel_format !== RAW_PIXEL_FORMAT) {
    throw new Error("manifest does not describe 320x180 BGRA frames");
  }
  if (root.encoding_frame_rate !== 60) throw new Error("encoding_frame_rate must be 60");
  requireString(root.scenario_id, "scenario_id");
  requireString(root.run_nonce, "run_nonce");
  requirePositiveInteger(root.process_id, "process_id");
  requireHash(root.capture_token_sha256, "capture_token_sha256");
  const start = requireUnsignedInteger(root.start_state_index, "start_state_index");
  const end = requireUnsignedInteger(root.end_state_index, "end_state_index");
  if (end < start) throw new Error("manifest state range is reversed");
  requireInteger(root.latest_state_index, "latest_state_index");
  requireBoolean(root.final_state_presented, "final_state_presented");
  const repeatCount = requireUnsignedInteger(
    root.repeated_presentation_count,
    "repeated_presentation_count",
  );
  requireRanges(root.unpresented_update_ranges, "unpresented_update_ranges");
  if (!Array.isArray(root.frames)) throw new Error("frames must be an array");

  let previousState = -1;
  let previousTimestamp = -1;
  let actualRepeatCount = 0;
  const frames = root.frames.map((value, index) => {
    const frame = requireRecord(value, `frames[${index}]`);
    if (frame.render_index !== index) throw new Error(`frames[${index}] render_index is not sequential`);
    const stateIndex = requireUnsignedInteger(frame.state_index, `frames[${index}].state_index`);
    const timestamp = requireUnsignedInteger(frame.timestamp_ns, `frames[${index}].timestamp_ns`);
    if (stateIndex < previousState) throw new Error(`frames[${index}] state_index moved backwards`);
    if (timestamp < previousTimestamp) throw new Error(`frames[${index}] timestamp moved backwards`);
    const repeated = requireBoolean(
      frame.repeated_state_presentation,
      `frames[${index}].repeated_state_presentation`,
    );
    if (repeated !== (stateIndex === previousState)) {
      throw new Error(`frames[${index}] repeated-state marker is inconsistent`);
    }
    if (repeated) actualRepeatCount++;
    previousState = stateIndex;
    previousTimestamp = timestamp;
    const relativePath = requireString(frame.path, `frames[${index}].path`);
    if (path.isAbsolute(relativePath)) throw new Error(`frames[${index}].path must be relative`);
    const parsed: PresentationFrame = {
      render_index: index,
      state_index: stateIndex,
      timestamp_ns: timestamp,
      path: relativePath,
      sha256: requireHash(frame.sha256, `frames[${index}].sha256`),
      bytes: requireUnsignedInteger(frame.bytes, `frames[${index}].bytes`),
      repeated_state_presentation: repeated,
      ...(frame.unpresented_updates_before === undefined
        ? {}
        : {
          unpresented_updates_before: requireRange(
            frame.unpresented_updates_before,
            `frames[${index}].unpresented_updates_before`,
          ),
        }),
    };
    return parsed;
  });
  if (actualRepeatCount !== repeatCount) {
    throw new Error("repeated_presentation_count does not match frames");
  }

  return {
    ...(root as unknown as PresentationManifest),
    frames,
  };
}

export async function canonicalDirectory(directory: string): Promise<string> {
  if (!path.isAbsolute(directory)) throw new Error("recording root must be absolute");
  const canonical = await realpath(directory);
  const info = await stat(canonical);
  if (!info.isDirectory()) throw new Error("recording root is not a directory");
  return canonical;
}

export function ensurePathInside(root: string, candidate: string, label: string): void {
  const relative = path.relative(root, candidate);
  if (relative === "" || (!relative.startsWith(`..${path.sep}`) && relative !== ".." && !path.isAbsolute(relative))) {
    return;
  }
  throw new Error(`${label} escapes the recording root`);
}

function requireRecord(value: unknown, label: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${label} must be an object`);
  }
  return value as Record<string, unknown>;
}

function requireString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.length === 0) throw new Error(`${label} must be a string`);
  return value;
}

function requireHash(value: unknown, label: string): string {
  const hash = requireString(value, label);
  if (!/^[a-f0-9]{64}$/.test(hash)) throw new Error(`${label} must be lowercase SHA-256`);
  return hash;
}

function requireInteger(value: unknown, label: string): number {
  if (!Number.isSafeInteger(value)) throw new Error(`${label} must be an integer`);
  return value as number;
}

function requireUnsignedInteger(value: unknown, label: string): number {
  const integer = requireInteger(value, label);
  if (integer < 0) throw new Error(`${label} must not be negative`);
  return integer;
}

function requirePositiveInteger(value: unknown, label: string): number {
  const integer = requireUnsignedInteger(value, label);
  if (integer === 0) throw new Error(`${label} must be positive`);
  return integer;
}

function requireBoolean(value: unknown, label: string): boolean {
  if (typeof value !== "boolean") throw new Error(`${label} must be a boolean`);
  return value;
}

function requireRange(value: unknown, label: string): StateIndexRange {
  const range = requireRecord(value, label);
  const start = requireUnsignedInteger(range.start_state_index, `${label}.start_state_index`);
  const end = requireUnsignedInteger(range.end_state_index, `${label}.end_state_index`);
  if (end < start) throw new Error(`${label} is reversed`);
  return { start_state_index: start, end_state_index: end };
}

function requireRanges(value: unknown, label: string): StateIndexRange[] {
  if (!Array.isArray(value)) throw new Error(`${label} must be an array`);
  return value.map((range, index) => requireRange(range, `${label}[${index}]`));
}
