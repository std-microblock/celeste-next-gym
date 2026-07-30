import { spawn, type ChildProcess } from "node:child_process";
import { once } from "node:events";
import { connect } from "node:net";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  createRunContext,
  reserveLoopbackPort,
  terminateOwnedProcess,
  updateRunManifest,
  validateGameInstall,
  waitForOwnedEverest,
  waitForProcessIdentity,
} from "./isolation/index.js";
import { captureCommand } from "./runtime/commands.js";
import {
  createHarnessPaths,
  prepareMods,
  prepareTrainingMod,
} from "./runtime/prepare-mods.js";
import type { ProcessIdentity } from "./types.js";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const paths = createHarnessPaths(repoRoot);
const gameInstall = validateGameInstall({
  repoRoot,
  gameRoot: paths.gameRoot,
  steamRoots: [],
});
const modPort = await reserveLoopbackPort();
const httpPort = await reserveLoopbackPort();
const context = createRunContext({
  repoRoot,
  gameInstall,
  modPort: modPort.port,
  httpPort: httpPort.port,
  git: {
    branch: captureCommand("git", ["branch", "--show-current"], repoRoot),
    head: captureCommand("git", ["rev-parse", "HEAD"], repoRoot),
  },
});
let game: ChildProcess | undefined;
let identity: ProcessIdentity | undefined;
let cleanupRequested = false;

try {
  prepareMods(paths, gameInstall);
  prepareTrainingMod(paths, gameInstall);
  await modPort.release();
  updateRunManifest(context, { status: "starting-training-preview" });
  game = spawn(gameInstall.executable, ["--disable-splash", "--loglevel", "info"], {
    cwd: gameInstall.gameRoot,
    windowsHide: false,
    stdio: "ignore",
    shell: false,
    env: {
      ...process.env,
      CELESTE_GYM_COLLECTOR_PORT: String(modPort.port),
      CELESTE_GYM_RUN_NONCE: context.runNonce,
      CELESTE_GYM_PREVIEW_FULLSCREEN: "1",
      EVEREST_SAVEPATH: context.saveRoot,
      EVEREST_TMPDIR: context.tempRoot,
    },
  });
  if (!game.pid) throw new Error("Celeste child did not expose a process id");
  identity = await waitForProcessIdentity(game.pid, gameInstall.executable);
  updateRunManifest(context, { status: "waiting-for-everest", game_process: identity });
  const ping = await waitForOwnedEverest(modPort.port, {
    runNonce: context.runNonce,
    processId: game.pid,
    port: modPort.port,
  });
  updateRunManifest(context, { status: "loading-training-world", everest_ping: ping });
  await loadTrainingWorld(modPort.port);
  await new Promise<void>((resolveWait) => setTimeout(resolveWait, 2_000));
  if (game.exitCode !== null) throw new Error(`Celeste exited during preview startup with ${game.exitCode}`);
  const gameLog = resolve(gameInstall.gameRoot, "log.txt");
  if (existsSync(gameLog)) {
    const log = readFileSync(gameLog, "utf8");
    const critical = log.lastIndexOf("ENCOUNTERED A CRITICAL ERROR");
    if (critical >= 0) throw new Error(`Celeste reported a critical error after loading the preview:\n${log.slice(critical, critical + 2_000)}`);
  }
  updateRunManifest(context, {
    status: "preview-ready",
    preview: { area_sid: "CelesteGymTraining/Training", room: "untitled-room" },
  });
  console.log(`TRAINING_PREVIEW_READY ${context.manifestPath}`);

  const stop = async (): Promise<void> => {
    if (cleanupRequested) return;
    cleanupRequested = true;
    if (game && identity) {
      const terminated = await terminateOwnedProcess({ child: game, expectedIdentity: identity });
      updateRunManifest(context, { status: "preview-stopped", cleanup: { game_terminated: terminated } });
    }
  };
  process.once("SIGINT", () => void stop());
  process.once("SIGTERM", () => void stop());
  await once(game, "exit");
  if (!cleanupRequested) updateRunManifest(context, { status: "preview-closed-by-user" });
} catch (error) {
  updateRunManifest(context, { status: "preview-failed", error: String(error) });
  if (game && identity) {
    await terminateOwnedProcess({ child: game, expectedIdentity: identity });
  }
  throw error;
} finally {
  await Promise.allSettled([modPort.release(), httpPort.release()]);
}

async function loadTrainingWorld(port: number): Promise<void> {
  const request = JSON.stringify({
    command: "simulate_area",
    area_sid: "CelesteGymTraining/Training",
    room: "untitled-room",
    inputs: [],
    skip_transitions: true,
  });
  await new Promise<void>((resolveRequest, reject) => {
    const socket = connect(port, "127.0.0.1");
    let response = "";
    const timer = setTimeout(() => {
      socket.destroy();
      reject(new Error("training preview load timed out"));
    }, 30_000);
    socket.setEncoding("utf8");
    socket.once("connect", () => socket.write(`${request}\n`));
    socket.on("data", (chunk) => {
      response += chunk;
      const newline = response.indexOf("\n");
      if (newline < 0) return;
      clearTimeout(timer);
      socket.destroy();
      const parsed = JSON.parse(response.slice(0, newline)) as { success?: boolean; error?: string };
      if (parsed.success !== true) reject(new Error(parsed.error ?? "training preview load failed"));
      else resolveRequest();
    });
    socket.once("error", (error) => {
      clearTimeout(timer);
      reject(error);
    });
  });
}
