import { createHash } from "node:crypto";
import { readFile, realpath, rename, stat, writeFile } from "node:fs/promises";
import path from "node:path";

import type { ProcessIdentity } from "../types.js";
import type { EncodingResult } from "./encoder.js";
import {
  canonicalDirectory,
  ensurePathInside,
  type StateIndexRange,
} from "./manifest.js";

export interface RecordingArtifactEntry {
  readonly kind: "scenario-master" | "technique-clip";
  readonly scenario_id: string;
  readonly technique_id?: string;
  readonly state_span: StateIndexRange;
  readonly raw_manifest: FileIdentity;
  readonly trace: FileIdentity;
  readonly video: FileIdentity & EncodingResult["probe"];
  readonly poster?: FileIdentity;
  readonly media_processes: EncodingResult["processes"];
}

export interface FileIdentity {
  readonly path: string;
  readonly sha256: string;
  readonly bytes: number;
}

export async function createArtifactEntry(options: {
  readonly recordingRoot: string;
  readonly kind: RecordingArtifactEntry["kind"];
  readonly scenarioId: string;
  readonly techniqueId?: string;
  readonly stateSpan: StateIndexRange;
  readonly rawManifestPath: string;
  readonly tracePath: string;
  readonly encoding: EncodingResult;
}): Promise<RecordingArtifactEntry> {
  const root = await canonicalDirectory(options.recordingRoot);
  const rawManifest = await identifyFile(root, options.rawManifestPath);
  const trace = await identifyFile(root, options.tracePath);
  const video = await identifyFile(root, options.encoding.output_path);
  const poster = options.encoding.poster
    ? await identifyFile(root, options.encoding.poster.output_path)
    : undefined;
  return {
    kind: options.kind,
    scenario_id: options.scenarioId,
    ...(options.techniqueId ? { technique_id: options.techniqueId } : {}),
    state_span: options.stateSpan,
    raw_manifest: rawManifest,
    trace,
    video: { ...video, ...options.encoding.probe },
    ...(poster ? { poster } : {}),
    media_processes: options.encoding.processes,
  };
}

export async function writeArtifactManifest(options: {
  readonly recordingRoot: string;
  readonly runNonce: string;
  readonly gameProcess: ProcessIdentity;
  readonly artifacts: readonly RecordingArtifactEntry[];
}): Promise<string> {
  const root = await canonicalDirectory(options.recordingRoot);
  const manifestPath = path.join(root, "artifacts.json");
  const temporaryPath = `${manifestPath}.tmp`;
  const document = {
    schema_version: 1,
    run_nonce: options.runNonce,
    game_process: options.gameProcess,
    created_at: new Date().toISOString(),
    artifacts: options.artifacts,
  };
  await writeFile(
    temporaryPath,
    `${JSON.stringify(document, null, 2)}\n`,
    "utf8",
  );
  await rename(temporaryPath, manifestPath);
  return manifestPath;
}

async function identifyFile(
  root: string,
  candidate: string,
): Promise<FileIdentity> {
  const canonical = await realpath(candidate);
  ensurePathInside(root, canonical, "artifact");
  const info = await stat(canonical);
  if (!info.isFile())
    throw new Error(`artifact is not a regular file: ${canonical}`);
  const bytes = await readFile(canonical);
  return {
    path: path.relative(root, canonical).replaceAll("\\", "/"),
    sha256: createHash("sha256").update(bytes).digest("hex"),
    bytes: bytes.byteLength,
  };
}
