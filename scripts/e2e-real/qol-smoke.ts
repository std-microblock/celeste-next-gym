import { spawn, type ChildProcess } from "node:child_process";
import { cpSync, existsSync, readFileSync } from "node:fs";
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
  game = spawn(gameInstall.executable, ["--disable-splash", "--loglevel", "info"], {
    cwd: gameInstall.gameRoot,
    windowsHide: true,
    stdio: "ignore",
    shell: false,
    env: {
      ...process.env,
      CELESTE_GYM_COLLECTOR_PORT: String(modPort.port),
      CELESTE_GYM_RUN_NONCE: context.runNonce,
      EVEREST_SAVEPATH: context.saveRoot,
      EVEREST_TMPDIR: context.tempRoot,
    },
  });
  if (!game.pid) throw new Error("Celeste child did not expose a process id");
  identity = await waitForProcessIdentity(game.pid, gameInstall.executable);
  const ping = await waitForOwnedEverest(modPort.port, {
    runNonce: context.runNonce,
    processId: game.pid,
    port: modPort.port,
  });
  await new Promise<void>((resolveWait) => setTimeout(resolveWait, 2_000));

  const logPath = resolve(gameInstall.gameRoot, "log.txt");
  if (!existsSync(logPath)) throw new Error("Celeste log.txt was not created");
  const log = readFileSync(logPath, "utf8");
  const critical = log.lastIndexOf("ENCOUNTERED A CRITICAL ERROR");
  if (critical >= 0) throw new Error(log.slice(critical, critical + 4_000));
  if (!log.includes("Loading microblock's QoL Utils")) {
    throw new Error("MicroblocksQolUtils load marker was not found in log.txt");
  }
  updateRunManifest(context, { status: "qol-smoke-passed", everest_ping: ping });
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

