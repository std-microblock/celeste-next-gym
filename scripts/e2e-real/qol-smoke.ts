import { spawn, type ChildProcess } from "node:child_process";
import { cpSync, existsSync, readFileSync, statSync } from "node:fs";
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
import { captureCommand, runCommand } from "./runtime/commands.js";
import {
  createHarnessPaths,
  prepareMods,
  removeValidatedTarget,
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
try {
  runCommand(process.execPath, [resolve(repoRoot, "scripts/build-qol-mod.mjs")], repoRoot);
  prepareMods(paths, gameInstall);
  const installed = resolve(gameInstall.gameRoot, "Mods", "MicroblocksQolUtils");
  removeValidatedTarget(installed, resolve(gameInstall.gameRoot, "Mods"));
  cpSync(resolve(repoRoot, "mods", "MicroblocksQolUtils", "Build"), installed, {
    recursive: true,
  });

  await modPort.release();
  updateRunManifest(context, { status: "starting-qol-smoke" });
  const captureOutput = resolve(context.tempRoot, "qol-capture-smoke.mkv");
  game = spawn(gameInstall.executable, ["--disable-splash", "--loglevel", "info"], {
    cwd: gameInstall.gameRoot,
    // WGC cannot create a GraphicsCaptureItem for a deliberately hidden game window.
    // This smoke is the one E2E that must display Celeste to exercise real capture.
    windowsHide: false,
    stdio: "ignore",
    shell: false,
    env: {
      ...process.env,
      CELESTE_GYM_COLLECTOR_PORT: String(modPort.port),
      CELESTE_GYM_RUN_NONCE: context.runNonce,
      EVEREST_SAVEPATH: context.saveRoot,
      EVEREST_TMPDIR: context.tempRoot,
      MICROBLOCKS_QOL_CAPTURE_SMOKE_OUTPUT: captureOutput,
    },
  });
  if (!game.pid) throw new Error("Celeste child did not expose a process id");
  identity = await waitForProcessIdentity(game.pid, gameInstall.executable);
  const ping = await waitForOwnedEverest(modPort.port, {
    runNonce: context.runNonce,
    processId: game.pid,
    port: modPort.port,
  });
  const logPath = resolve(gameInstall.gameRoot, "log.txt");
  const deadline = Date.now() + 15_000;
  let log = "";
  while (Date.now() < deadline) {
    if (existsSync(logPath)) log = readFileSync(logPath, "utf8");
    if (log.includes("QOL_CAPTURE_SMOKE_PASSED")
        && existsSync(captureOutput)
        && statSync(captureOutput).size >= 1_000) break;
    await new Promise<void>((resolveWait) => setTimeout(resolveWait, 250));
  }
  if (!existsSync(logPath)) throw new Error("Celeste log.txt was not created");
  const critical = log.lastIndexOf("ENCOUNTERED A CRITICAL ERROR");
  if (critical >= 0) throw new Error(log.slice(critical, critical + 4_000));
  if (!log.includes("Loading microblock's QoL Utils")) {
    throw new Error("MicroblocksQolUtils load marker was not found in log.txt");
  }
  if (!log.includes("QOL_CAPTURE_SMOKE_PASSED")) {
    throw new Error("Native scap/FFmpeg capture smoke marker was not found in log.txt");
  }
  if (!existsSync(captureOutput) || statSync(captureOutput).size < 1_000) {
    throw new Error("Native scap/FFmpeg capture smoke output was not created");
  }
  updateRunManifest(context, {
    status: "qol-smoke-passed",
    everest_ping: ping,
    native_capture_output: captureOutput,
  });
  console.log(`QOL_SMOKE_PASSED ${context.manifestPath}`);
} catch (error) {
  updateRunManifest(context, { status: "qol-smoke-failed", error: String(error) });
  throw error;
} finally {
  if (game && identity) {
    await terminateOwnedProcess({ child: game, expectedIdentity: identity });
  }
  await Promise.allSettled([modPort.release(), httpPort.release()]);
}
