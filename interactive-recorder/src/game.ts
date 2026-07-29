import { createHash, randomUUID } from "node:crypto";
import { spawn, type ChildProcess } from "node:child_process";
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import net from "node:net";
import { dirname, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createInterface } from "node:readline/promises";

import {
  createRunContext,
  reserveLoopbackPort,
  terminateOwnedProcess,
  updateRunManifest,
  validateGameInstall,
  waitForOwnedEverest,
  waitForProcessIdentity,
} from "../../scripts/e2e-real/isolation/index.js";
import type { ProcessIdentity } from "../../scripts/e2e-real/types.js";
import { captureCommand } from "../../scripts/e2e-real/runtime/commands.js";
import {
  createHarnessPaths,
  prepareMods,
} from "../../scripts/e2e-real/runtime/prepare-mods.js";

interface InteractiveStatus {
  readonly state: "loading" | "active" | "stopped";
  readonly recording_id: string;
  readonly area_sid: string;
  readonly room: string;
  readonly frame_count: number;
  readonly state_count: number;
  readonly trace_path?: string;
  readonly reason?: string;
}

interface CollectorResponse {
  readonly success?: boolean;
  readonly error?: string;
  readonly interactive_recording?: InteractiveStatus;
}

export async function recordGame(
  options: { readonly maxFrames?: number } = {},
): Promise<string> {
  const recorderRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
  const repoRoot = resolve(recorderRoot, "..");
  const mainWorkspaceLine = captureCommand(
    "git",
    ["worktree", "list", "--porcelain"],
    repoRoot,
  )
    .split(/\r?\n/)
    .find((line) => line.startsWith("worktree "));
  const mainWorkspace = mainWorkspaceLine?.slice("worktree ".length);
  if (!mainWorkspace || resolve(mainWorkspace) !== repoRoot) {
    throw new Error(
      `interactive game recording must run from the main workspace: ${mainWorkspace ?? "(unknown)"}`,
    );
  }
  const paths = createHarnessPaths(repoRoot);
  const gameInstall = validateGameInstall({
    repoRoot,
    gameRoot: paths.gameRoot,
  });
  const git = {
    branch: captureCommand("git", ["branch", "--show-current"], repoRoot),
    head: captureCommand("git", ["rev-parse", "HEAD"], repoRoot),
  };
  if (!git.branch)
    throw new Error(
      "interactive game recording requires a named branch, not detached HEAD",
    );
  const dirty = captureCommand("git", ["status", "--porcelain"], repoRoot);
  if (dirty)
    throw new Error(
      `interactive game recording requires a clean main workspace:\n${dirty}`,
    );
  const modPort = await reserveLoopbackPort();
  const unusedHttpPort = await reserveLoopbackPort();
  const context = createRunContext({
    repoRoot,
    gameInstall,
    modPort: modPort.port,
    httpPort: unusedHttpPort.port,
    git,
  });
  const recordingId = `manual-${new Date().toISOString().replaceAll(":", "-").replaceAll(".", "-")}`;
  const recordingRoot = resolve(
    recorderRoot,
    "recordings",
    `${recordingId}-${randomUUID()}`,
  );
  mkdirSync(recordingRoot, { recursive: true });
  const token =
    randomUUID().replaceAll("-", "") + randomUUID().replaceAll("-", "");
  let game: ChildProcess | undefined;
  let identity: ProcessIdentity | undefined;
  let runError: unknown;
  try {
    prepareMods(paths, gameInstall);
    await modPort.release();
    await unusedHttpPort.release();
    updateRunManifest(context, {
      status: "starting-interactive-game",
      recording_root: recordingRoot,
    });
    game = spawn(
      gameInstall.executable,
      ["--disable-splash", "--loglevel", "info"],
      {
        cwd: gameInstall.gameRoot,
        windowsHide: false,
        stdio: "ignore",
        shell: false,
        env: {
          ...process.env,
          CELESTE_GYM_COLLECTOR_PORT: String(modPort.port),
          CELESTE_GYM_RUN_NONCE: context.runNonce,
          CELESTE_GYM_RECORDING_ROOT: recordingRoot,
          EVEREST_SAVEPATH: context.saveRoot,
          EVEREST_TMPDIR: context.tempRoot,
        },
      },
    );
    if (!game.pid) throw new Error("Celeste child did not expose a process id");
    identity = await waitForProcessIdentity(game.pid, gameInstall.executable);
    updateRunManifest(context, {
      status: "waiting-for-everest",
      game_process: identity,
    });
    await waitForOwnedEverest(modPort.port, {
      runNonce: context.runNonce,
      processId: identity.processId,
      port: modPort.port,
    });
    const authenticated = {
      run_nonce: context.runNonce,
      process_id: identity.processId,
      capture_token: token,
    };
    await sendInteractive(modPort.port, {
      command: "interactive_start",
      ...authenticated,
      recording_id: recordingId,
      area_sid: "CelesteGymPlayground/Playground",
      room: "playground",
      max_frames: options.maxFrames ?? 36_000,
    });
    let status = await waitUntilActive(modPort.port, authenticated, game);
    updateRunManifest(context, {
      status: "interactive-recording",
      interactive_recording: status,
    });
    console.log(
      "Playground 已进入并开始逐 Player.Update 录制。回到此终端按 Enter 停止并保存。",
    );
    const prompt = createInterface({
      input: process.stdin,
      output: process.stdout,
    });
    try {
      await Promise.race([
        prompt.question(""),
        new Promise<never>((_, reject) =>
          game!.once("exit", () =>
            reject(
              new Error("Celeste exited before the recording was stopped"),
            ),
          ),
        ),
      ]);
    } finally {
      prompt.close();
    }
    status = await sendInteractive(modPort.port, {
      command: "interactive_stop",
      ...authenticated,
      reason: "user_finished",
    });
    if (status.state !== "stopped" || !status.trace_path)
      throw new Error("game did not finalize an interactive trace");
    const rawTrace = containedPath(recordingRoot, status.trace_path);
    const mapOutput = resolve(
      recordingRoot,
      "maps",
      "CelesteGymPlayground",
      "Playground.bin",
    );
    const fixtureOutput = resolve(
      recordingRoot,
      "maps",
      "playground.map.fixture.json",
    );
    mkdirSync(dirname(mapOutput), { recursive: true });
    copyFileSync(
      resolve(recorderRoot, "maps", "CelesteGymPlayground", "Playground.bin"),
      mapOutput,
    );
    copyFileSync(
      resolve(recorderRoot, "maps", "playground.map.fixture.json"),
      fixtureOutput,
    );
    const mapHash = createHash("sha256")
      .update(readFileSync(mapOutput))
      .digest("hex");
    const trace = JSON.parse(readFileSync(rawTrace, "utf8")) as {
      map: Record<string, unknown>;
    };
    trace.map.sha256 = mapHash;
    const traceOutput = resolve(recordingRoot, "trace.json");
    writeFileSync(traceOutput, `${JSON.stringify(trace, null, 2)}\n`, "utf8");
    writeFileSync(
      resolve(recordingRoot, "manifest.json"),
      `${JSON.stringify(
        {
          version: 1,
          trace: "trace.json",
          map: "maps/CelesteGymPlayground/Playground.bin",
          map_fixture: "maps/playground.map.fixture.json",
          map_sha256: mapHash,
          game_process: identity,
          run_manifest: context.manifestPath,
          frames: status.frame_count,
        },
        null,
        2,
      )}\n`,
      "utf8",
    );
    updateRunManifest(context, {
      status: "completed",
      trace: traceOutput,
      frames: status.frame_count,
    });
    console.log(
      `已保存 ${status.frame_count} 个输入帧和 ${status.state_count} 个状态：${traceOutput}`,
    );
    return traceOutput;
  } catch (error) {
    runError = error;
    updateRunManifest(context, { status: "failed", error: String(error) });
    throw error;
  } finally {
    await Promise.allSettled([modPort.release(), unusedHttpPort.release()]);
    const terminated = cleanupOwned(game, identity);
    updateRunManifest(context, {
      status: runError ? "failed-cleanup-finished" : "cleanup-finished",
      game_terminated: terminated,
    });
  }
}

async function waitUntilActive(
  port: number,
  authenticated: Record<string, unknown>,
  game: ChildProcess,
): Promise<InteractiveStatus> {
  const deadline = Date.now() + 30_000;
  do {
    if (game.exitCode !== null)
      throw new Error("Celeste exited while loading Playground");
    const status = await sendInteractive(port, {
      command: "interactive_status",
      ...authenticated,
    });
    if (status.state === "active") return status;
    await new Promise<void>((resolveWait) => setTimeout(resolveWait, 100));
  } while (Date.now() < deadline);
  throw new Error(
    "Playground player did not become recordable within 30 seconds",
  );
}

async function sendInteractive(
  port: number,
  request: unknown,
): Promise<InteractiveStatus> {
  const response = await new Promise<CollectorResponse>(
    (resolveResponse, reject) => {
      const socket = net.createConnection({ host: "127.0.0.1", port });
      let data = "";
      let settled = false;
      const finish = (callback: () => void) => {
        if (settled) return;
        settled = true;
        clearTimeout(timer);
        socket.destroy();
        callback();
      };
      const timer = setTimeout(
        () =>
          finish(() =>
            reject(new Error("interactive collector request timed out")),
          ),
        60_000,
      );
      timer.unref();
      socket.setEncoding("utf8");
      socket.once("connect", () =>
        socket.write(`${JSON.stringify(request)}\n`),
      );
      socket.on("data", (chunk) => {
        data += chunk;
        const newline = data.indexOf("\n");
        if (newline < 0) return;
        finish(() => {
          try {
            resolveResponse(
              JSON.parse(data.slice(0, newline)) as CollectorResponse,
            );
          } catch (error) {
            reject(
              new Error(
                `invalid interactive collector response: ${String(error)}`,
              ),
            );
          }
        });
      });
      socket.once("error", (error) => finish(() => reject(error)));
      socket.once("end", () =>
        finish(() =>
          reject(new Error("interactive collector closed without a response")),
        ),
      );
    },
  );
  if (response.success !== true || !response.interactive_recording) {
    throw new Error(
      response.error ?? "interactive collector returned no status",
    );
  }
  return response.interactive_recording;
}

function containedPath(root: string, relativePath: string): string {
  const candidate = resolve(root, relativePath);
  const rel = relative(root, candidate);
  if (rel.startsWith("..") || resolve(root, rel) !== candidate)
    throw new Error("trace path escapes recording root");
  return candidate;
}

function cleanupOwned(
  game: ChildProcess | undefined,
  identity: ProcessIdentity | undefined,
): boolean {
  if (!identity) {
    if (game?.pid && game.exitCode === null)
      console.warn(
        `refusing to terminate PID ${game.pid}: no recorded process identity`,
      );
    return false;
  }
  return terminateOwnedProcess({ child: game, expectedIdentity: identity });
}
