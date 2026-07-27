import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  stat,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, it } from "node:test";
import {
  cleanupPartialArtifact,
  encodePresentationRecording,
  resolveExecutable,
  runLavfiSmokeTest,
} from "./encoder.js";
import {
  RAW_FRAME_BYTES,
  loadPresentationManifest,
  type PresentationFrame,
  type PresentationManifest,
} from "./manifest.js";

const temporaryRoots: string[] = [];
const ffmpegPath = process.env.FFMPEG_PATH;
const ffprobePath = process.env.FFPROBE_PATH;
const hasMediaTools = Boolean(ffmpegPath && ffprobePath);

afterEach(async () => {
  await Promise.all(temporaryRoots.splice(0).map((root) => rm(root, {
    recursive: true,
    force: true,
  })));
});

describe("presentation recording artifacts", () => {
  it("validates state gaps, duplicate presentations, hashes, and containment", async () => {
    const fixture = await createFixture();
    const loaded = await loadPresentationManifest(fixture.root, fixture.manifestPath);

    assert.equal(loaded.framePaths.length, 3);
    assert.equal(loaded.manifest.frames[1]?.repeated_state_presentation, true);
    assert.deepEqual(loaded.manifest.unpresented_update_ranges, [{
      start_state_index: 1,
      end_state_index: 2,
    }]);
  });

  it("rejects frame paths that escape the session directory", async () => {
    const fixture = await createFixture();
    const manifest = JSON.parse(await readFile(fixture.manifestPath, "utf8")) as PresentationManifest;
    manifest.frames[0]!.path = "../../outside.bgra";
    await writeFile(fixture.manifestPath, JSON.stringify(manifest));

    await assert.rejects(
      () => loadPresentationManifest(fixture.root, fixture.manifestPath),
      /escapes/,
    );
  });

  it("requires explicit absolute media tool paths", async () => {
    await assert.rejects(() => resolveExecutable("ffmpeg", "ffmpeg"), /absolute/);
  });

  it("encodes BGRA presentation frames to probed H.264 CFR60", {
    skip: !hasMediaTools,
  }, async () => {
    const fixture = await createFixture();
    const outputPath = path.join(fixture.root, "videos", "scenario.mp4");
    const result = await encodePresentationRecording({
      recordingRoot: fixture.root,
      manifestPath: fixture.manifestPath,
      outputPath,
      ffmpegPath: ffmpegPath!,
      ffprobePath: ffprobePath!,
    });

    assert.equal(result.probe.codec, "h264");
    assert.equal(result.probe.pixel_format, "yuv420p");
    assert.equal(result.probe.frame_rate, "60/1");
    assert.equal(result.probe.frame_count, 3);
    assert.equal(result.processes.length, 2);
    assert.ok(result.processes.every((identity) => identity.pid > 0));
    assert.ok((await stat(outputPath)).isFile());
    await assert.rejects(() => stat(outputPath.replace(/\.mp4$/, ".partial.mp4")));
  });

  it("runs a no-game lavfi encoder smoke test", {
    skip: !hasMediaTools,
  }, async () => {
    const root = await createRoot();
    const result = await runLavfiSmokeTest({
      recordingRoot: root,
      outputPath: path.join(root, "lavfi.mp4"),
      ffmpegPath: ffmpegPath!,
      ffprobePath: ffprobePath!,
    });

    assert.equal(result.probe.codec, "h264");
    assert.equal(result.probe.width, 320);
    assert.equal(result.probe.height, 180);
  });

  it("retains a failed partial until bounded explicit cleanup", {
    skip: !hasMediaTools,
  }, async () => {
    const fixture = await createFixture();
    const outputPath = path.join(fixture.root, "failed.mp4");
    const partialPath = path.join(fixture.root, "failed.partial.mp4");
    await assert.rejects(() => encodePresentationRecording({
      recordingRoot: fixture.root,
      manifestPath: fixture.manifestPath,
      outputPath,
      ffmpegPath: ffmpegPath!,
      // ffmpeg is intentionally not ffprobe, so probing the valid partial fails.
      ffprobePath: ffmpegPath!,
    }));
    assert.ok((await stat(partialPath)).isFile());
    await assert.rejects(() => stat(outputPath));

    await cleanupPartialArtifact(fixture.root, partialPath);
    await assert.rejects(() => stat(partialPath));
    await assert.rejects(
      () => cleanupPartialArtifact(fixture.root, path.join(fixture.root, "..", "escape.partial.mp4")),
      /escapes/,
    );
  });
});

async function createFixture(): Promise<{ root: string; manifestPath: string }> {
  const root = await createRoot();
  const session = path.join(root, "scenarios", "scenario-1", "token");
  const framesDirectory = path.join(session, "frames");
  await mkdir(framesDirectory, { recursive: true });
  const stateIndices = [0, 0, 3];
  const frames: PresentationFrame[] = [];
  for (let renderIndex = 0; renderIndex < stateIndices.length; renderIndex++) {
    const bytes = Buffer.alloc(RAW_FRAME_BYTES);
    for (let offset = 0; offset < bytes.length; offset += 4) {
      bytes[offset] = renderIndex * 80;
      bytes[offset + 1] = 255 - renderIndex * 80;
      bytes[offset + 2] = 40;
      bytes[offset + 3] = 255;
    }
    const framePath = path.join(framesDirectory, `${renderIndex.toString().padStart(6, "0")}.bgra`);
    await writeFile(framePath, bytes);
    frames.push({
      render_index: renderIndex,
      state_index: stateIndices[renderIndex]!,
      timestamp_ns: renderIndex * 16_666_700,
      path: path.relative(session, framePath).replaceAll("\\", "/"),
      sha256: createHash("sha256").update(bytes).digest("hex"),
      bytes: bytes.byteLength,
      repeated_state_presentation: renderIndex === 1,
      ...(renderIndex === 2
        ? { unpresented_updates_before: { start_state_index: 1, end_state_index: 2 } }
        : {}),
    });
  }
  const manifest: PresentationManifest = {
    schema_version: 1,
    capture_semantics: "presentation_frames",
    scenario_id: "scenario-1",
    run_nonce: "run-nonce",
    process_id: 42,
    capture_token_sha256: "a".repeat(64),
    width: 320,
    height: 180,
    pixel_format: "bgra",
    encoding_frame_rate: 60,
    started_at: new Date(0).toISOString(),
    finalized_at: new Date(100).toISOString(),
    outcome: "ready",
    start_state_index: 0,
    end_state_index: 3,
    latest_state_index: 3,
    final_state_presented: true,
    repeated_presentation_count: 1,
    unpresented_update_ranges: [{ start_state_index: 1, end_state_index: 2 }],
    frames,
  };
  const manifestPath = path.join(session, "manifest.json");
  await writeFile(manifestPath, JSON.stringify(manifest));
  return { root, manifestPath };
}

async function createRoot(): Promise<string> {
  const root = await mkdtemp(path.join(tmpdir(), "celeste-gym-recording-"));
  temporaryRoots.push(root);
  return root;
}
