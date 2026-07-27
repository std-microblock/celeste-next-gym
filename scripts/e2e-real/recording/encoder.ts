import { createHash } from "node:crypto";
import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import {
  lstat,
  mkdir,
  readFile,
  realpath,
  rename,
  rm,
  stat,
} from "node:fs/promises";
import path from "node:path";
import { once } from "node:events";
import {
  RAW_FRAME_BYTES,
  RAW_HEIGHT,
  RAW_WIDTH,
  canonicalDirectory,
  ensurePathInside,
  loadPresentationManifest,
} from "./manifest.js";

export interface ToolProcessIdentity {
  pid: number;
  executable_path: string;
  spawned_at: string;
  exited_at: string;
  exit_code: number;
}

export interface VideoProbe {
  codec: string;
  pixel_format: string;
  width: number;
  height: number;
  frame_rate: string;
  frame_count?: number;
  duration_seconds?: number;
}

export interface EncodingResult {
  output_path: string;
  sha256: string;
  bytes: number;
  probe: VideoProbe;
  processes: ToolProcessIdentity[];
  poster?: { output_path: string; sha256: string; bytes: number };
}

export interface EncodePresentationOptions {
  recordingRoot: string;
  manifestPath: string;
  outputPath: string;
  ffmpegPath: string;
  ffprobePath: string;
  stateWindow?: { readonly startStateIndex: number; readonly endStateIndex: number };
  posterPath?: string;
}

export async function encodePresentationRecording(
  options: EncodePresentationOptions,
): Promise<EncodingResult> {
  const recordingRoot = await canonicalDirectory(options.recordingRoot);
  const loaded = await loadPresentationManifest(recordingRoot, options.manifestPath);
  const selectedFrames = selectFrames(loaded, options.stateWindow);
  if (selectedFrames.length === 0) throw new Error("cannot encode an empty presentation recording window");
  const outputPath = await validateNewOutputPath(recordingRoot, options.outputPath);
  const partialPath = partialMp4Path(outputPath);
  await assertMissing(outputPath, "output");
  await assertMissing(partialPath, "partial output");
  const ffmpeg = await resolveExecutable(options.ffmpegPath, "ffmpeg");
  const ffprobe = await resolveExecutable(options.ffprobePath, "ffprobe");
  const processes: ToolProcessIdentity[] = [];

  const ffmpegRun = startTool(ffmpeg, [
    "-hide_banner",
    "-nostdin",
    "-loglevel",
    "error",
    "-f",
    "rawvideo",
    "-pixel_format",
    "bgra",
    "-video_size",
    `${RAW_WIDTH}x${RAW_HEIGHT}`,
    "-framerate",
    "60",
    "-i",
    "pipe:0",
    "-an",
    "-c:v",
    "libx264",
    "-crf",
    "18",
    "-pix_fmt",
    "yuv420p",
    "-r",
    "60",
    "-movflags",
    "+faststart",
    "-n",
    partialPath,
  ]);
  try {
    for (const framePath of selectedFrames) {
      const frame = await readFile(framePath);
      if (frame.byteLength !== RAW_FRAME_BYTES) {
        throw new Error(`raw frame changed size during encoding: ${framePath}`);
      }
      if (!ffmpegRun.child.stdin.write(frame)) {
        await once(ffmpegRun.child.stdin, "drain");
      }
    }
    ffmpegRun.child.stdin.end();
  } catch (error) {
    ffmpegRun.child.stdin.destroy();
    ffmpegRun.child.kill();
    await ffmpegRun.completion.catch(() => undefined);
    throw error;
  }
  processes.push(await ffmpegRun.completion);

  const probeRun = await probeVideo(ffprobe, partialPath);
  processes.push(probeRun.identity);
  validateProbe(probeRun.probe);
  await rename(partialPath, outputPath);
  const bytes = await readFile(outputPath);
  const poster = options.posterPath
    ? await createPoster(recordingRoot, ffmpeg, outputPath, options.posterPath, processes)
    : undefined;
  return {
    output_path: outputPath,
    sha256: createHash("sha256").update(bytes).digest("hex"),
    bytes: bytes.byteLength,
    probe: probeRun.probe,
    processes,
    ...(poster ? { poster } : {}),
  };
}

function selectFrames(
  loaded: Awaited<ReturnType<typeof loadPresentationManifest>>,
  window: EncodePresentationOptions['stateWindow'],
): string[] {
  if (!window) return loaded.framePaths;
  if (!Number.isSafeInteger(window.startStateIndex) || !Number.isSafeInteger(window.endStateIndex)
    || window.startStateIndex < 0 || window.endStateIndex < window.startStateIndex) {
    throw new Error('invalid recording state window');
  }
  return loaded.manifest.frames.flatMap((frame, index) =>
    frame.state_index >= window.startStateIndex && frame.state_index <= window.endStateIndex
      ? [loaded.framePaths[index]!] : []);
}

async function createPoster(
  root: string,
  ffmpeg: string,
  videoPath: string,
  requestedPath: string,
  processes: ToolProcessIdentity[],
): Promise<{ output_path: string; sha256: string; bytes: number }> {
  const posterPath = await validateNewArtifactPath(root, requestedPath, '.png', 'poster');
  const partialPath = posterPath.slice(0, -4) + '.partial.png';
  await assertMissing(posterPath, 'poster');
  await assertMissing(partialPath, 'partial poster');
  const run = startTool(ffmpeg, ['-hide_banner', '-nostdin', '-loglevel', 'error', '-i', videoPath, '-frames:v', '1', '-n', partialPath]);
  run.child.stdin.end();
  processes.push(await run.completion);
  await rename(partialPath, posterPath);
  const bytes = await readFile(posterPath);
  return { output_path: posterPath, sha256: createHash('sha256').update(bytes).digest('hex'), bytes: bytes.byteLength };
}

export async function runLavfiSmokeTest(options: {
  recordingRoot: string;
  outputPath: string;
  ffmpegPath: string;
  ffprobePath: string;
}): Promise<EncodingResult> {
  const recordingRoot = await canonicalDirectory(options.recordingRoot);
  const outputPath = await validateNewOutputPath(recordingRoot, options.outputPath);
  const partialPath = partialMp4Path(outputPath);
  await assertMissing(outputPath, "output");
  await assertMissing(partialPath, "partial output");
  const ffmpeg = await resolveExecutable(options.ffmpegPath, "ffmpeg");
  const ffprobe = await resolveExecutable(options.ffprobePath, "ffprobe");
  const processes: ToolProcessIdentity[] = [];
  const run = startTool(ffmpeg, [
    "-hide_banner",
    "-nostdin",
    "-loglevel",
    "error",
    "-f",
    "lavfi",
    "-i",
    `testsrc=size=${RAW_WIDTH}x${RAW_HEIGHT}:rate=60:duration=0.1`,
    "-an",
    "-c:v",
    "libx264",
    "-crf",
    "18",
    "-pix_fmt",
    "yuv420p",
    "-r",
    "60",
    "-movflags",
    "+faststart",
    "-n",
    partialPath,
  ]);
  run.child.stdin.end();
  processes.push(await run.completion);
  const probeRun = await probeVideo(ffprobe, partialPath);
  processes.push(probeRun.identity);
  validateProbe(probeRun.probe);
  await rename(partialPath, outputPath);
  const bytes = await readFile(outputPath);
  return {
    output_path: outputPath,
    sha256: createHash("sha256").update(bytes).digest("hex"),
    bytes: bytes.byteLength,
    probe: probeRun.probe,
    processes,
  };
}

export async function cleanupPartialArtifact(
  recordingRoot: string,
  partialPath: string,
): Promise<void> {
  const root = await canonicalDirectory(recordingRoot);
  if (!path.isAbsolute(partialPath) || !partialPath.endsWith(".partial.mp4")) {
    throw new Error("cleanup target must be an absolute .partial.mp4 path");
  }
  ensurePathInside(root, path.resolve(partialPath), "partial output");
  await rm(partialPath, { force: true, recursive: false });
}

export async function resolveExecutable(executablePath: string, label: string): Promise<string> {
  if (!path.isAbsolute(executablePath)) {
    throw new Error(`${label} path must be explicit and absolute`);
  }
  const linkInfo = await lstat(executablePath);
  if (linkInfo.isSymbolicLink()) throw new Error(`${label} path must not be a symbolic link`);
  const canonical = await realpath(executablePath);
  const info = await stat(canonical);
  if (!info.isFile()) throw new Error(`${label} path is not a regular file`);
  return canonical;
}

function startTool(executable: string, args: string[]): {
  child: ChildProcessWithoutNullStreams;
  completion: Promise<ToolProcessIdentity>;
} {
  const spawnedAt = new Date();
  const child = spawn(executable, args, {
    windowsHide: true,
    stdio: ["pipe", "pipe", "pipe"],
  });
  if (child.pid === undefined) throw new Error(`failed to spawn ${executable}`);
  const pid = child.pid;
  let stderr = "";
  child.stderr.setEncoding("utf8");
  child.stderr.on("data", (chunk: string) => {
    if (stderr.length < 1024 * 1024) stderr += chunk;
  });
  const completion = new Promise<ToolProcessIdentity>((resolve, reject) => {
    child.once("error", reject);
    child.once("exit", (code) => {
      const identity: ToolProcessIdentity = {
        pid,
        executable_path: executable,
        spawned_at: spawnedAt.toISOString(),
        exited_at: new Date().toISOString(),
        exit_code: code ?? -1,
      };
      if (code === 0) resolve(identity);
      else reject(new ToolExitError(executable, identity, stderr));
    });
  });
  return { child, completion };
}

async function probeVideo(
  ffprobe: string,
  videoPath: string,
): Promise<{ identity: ToolProcessIdentity; probe: VideoProbe }> {
  const run = startTool(ffprobe, [
    "-v",
    "error",
    "-select_streams",
    "v:0",
    "-show_entries",
    "stream=codec_name,pix_fmt,width,height,r_frame_rate,nb_frames,duration",
    "-of",
    "json",
    videoPath,
  ]);
  let stdout = "";
  run.child.stdout.setEncoding("utf8");
  run.child.stdout.on("data", (chunk: string) => {
    if (stdout.length < 1024 * 1024) stdout += chunk;
  });
  run.child.stdin.end();
  const identity = await run.completion;
  const parsed = JSON.parse(stdout) as {
    streams?: Array<Record<string, string | number>>;
  };
  const stream = parsed.streams?.[0];
  if (!stream) throw new Error("ffprobe returned no video stream");
  return {
    identity,
    probe: {
      codec: String(stream.codec_name ?? ""),
      pixel_format: String(stream.pix_fmt ?? ""),
      width: Number(stream.width),
      height: Number(stream.height),
      frame_rate: String(stream.r_frame_rate ?? ""),
      ...(stream.nb_frames === undefined
        ? {}
        : { frame_count: Number(stream.nb_frames) }),
      ...(stream.duration === undefined
        ? {}
        : { duration_seconds: Number(stream.duration) }),
    },
  };
}

function validateProbe(probe: VideoProbe): void {
  if (probe.codec !== "h264") throw new Error(`encoded codec is ${probe.codec}, expected h264`);
  if (probe.pixel_format !== "yuv420p") {
    throw new Error(`encoded pixel format is ${probe.pixel_format}, expected yuv420p`);
  }
  if (probe.width !== RAW_WIDTH || probe.height !== RAW_HEIGHT) {
    throw new Error(`encoded dimensions are ${probe.width}x${probe.height}`);
  }
  if (probe.frame_rate !== "60/1") {
    throw new Error(`encoded frame rate is ${probe.frame_rate}, expected 60/1`);
  }
}

async function validateNewOutputPath(root: string, outputPath: string): Promise<string> {
  return await validateNewArtifactPath(root, outputPath, '.mp4', 'output');
}

async function validateNewArtifactPath(root: string, outputPath: string, extension: string, label: string): Promise<string> {
  if (!path.isAbsolute(outputPath) || path.extname(outputPath).toLowerCase() !== extension) {
    throw new Error(`${label} path must be an absolute ${extension} path`);
  }
  const resolved = path.resolve(outputPath);
  ensurePathInside(root, resolved, label);
  await mkdir(path.dirname(resolved), { recursive: true });
  const canonicalParent = await realpath(path.dirname(resolved));
  ensurePathInside(root, canonicalParent, `${label} parent`);
  return path.join(canonicalParent, path.basename(resolved));
}

function partialMp4Path(outputPath: string): string {
  return outputPath.slice(0, -4) + ".partial.mp4";
}

async function assertMissing(candidate: string, label: string): Promise<void> {
  try {
    await lstat(candidate);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return;
    throw error;
  }
  throw new Error(`${label} already exists: ${candidate}`);
}

export class ToolExitError extends Error {
  constructor(
    executable: string,
    readonly identity: ToolProcessIdentity,
    readonly stderr: string,
  ) {
    super(`${path.basename(executable)} exited with ${identity.exit_code}: ${stderr.trim()}`);
    this.name = "ToolExitError";
  }
}
