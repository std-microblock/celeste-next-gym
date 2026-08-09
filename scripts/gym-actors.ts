import { randomUUID } from "node:crypto";
import { spawn, type ChildProcess } from "node:child_process";
import { mkdirSync, renameSync, writeFileSync } from "node:fs";
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
} from "./e2e-real/isolation/index.js";
import type {
  GitIdentity,
  ProcessIdentity,
  RunContext,
} from "./e2e-real/types.js";
import { EVEREST_READY_TIMEOUT_MS, SERVICE_PORT_TIMEOUT_MS } from "./e2e-real/constants.js";
import { captureCommand, waitForPort } from "./e2e-real/runtime/commands.js";
import {
  createHarnessPaths,
  prepareMods,
} from "./e2e-real/runtime/prepare-mods.js";
import { createCollectorClient } from "./e2e-real/runtime/collector-client.js";

interface ActorOptions {
  readonly count: number;
  readonly areaId: number;
  readonly areaSid?: string;
  readonly showWindows: boolean;
  readonly smoke: boolean;
}

interface RunningActor {
  readonly index: number;
  readonly context: RunContext;
  readonly gymUrl: string;
  readonly game: ChildProcess;
  readonly gameIdentity: ProcessIdentity;
  readonly service: ChildProcess;
  readonly serviceIdentity: ProcessIdentity;
}

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");

export function parseActorOptions(argv: readonly string[]): ActorOptions {
  let count = 1;
  let areaId = 1;
  let areaSid: string | undefined;
  let showWindows = false;
  let smoke = false;
  for (let index = 0; index < argv.length; index++) {
    const argument = argv[index];
    if (argument === "--actors") count = boundedInteger(argv[++index], argument, 1, 32);
    else if (argument === "--area-id") areaId = boundedInteger(argv[++index], argument, 0, 10_000);
    else if (argument === "--area-sid") areaSid = requiredValue(argv[++index], argument);
    else if (argument === "--show-windows") showWindows = true;
    else if (argument === "--smoke") smoke = true;
    else throw new Error(`unknown gym actor argument: ${argument}`);
  }
  return Object.freeze({
    count,
    areaId,
    ...(areaSid ? { areaSid } : {}),
    showWindows,
    smoke,
  });
}

export async function runActorLauncher(): Promise<void> {
  const options = parseActorOptions(process.argv.slice(2));
  const paths = createHarnessPaths(repoRoot);
  const gameInstall = validateGameInstall({
    repoRoot,
    gameRoot: paths.gameRoot,
    steamRoots: [],
  });
  const git: GitIdentity = {
    branch: captureCommand("git", ["branch", "--show-current"], repoRoot),
    head: captureCommand("git", ["rev-parse", "HEAD"], repoRoot),
  };

  // The physical repository-owned Mods directory is shared by actors, so build
  // and install once before any Celeste child starts reading it.
  prepareMods(paths, gameInstall);

  const supervisorRoot = resolve(
    repoRoot,
    ".tmp",
    "gym-actors",
    `${new Date().toISOString().replaceAll(":", "-")}-${process.pid}-${randomUUID()}`,
  );
  mkdirSync(supervisorRoot, { recursive: true });
  const supervisorManifest = resolve(supervisorRoot, "manifest.json");
  const actors: RunningActor[] = [];
  let shuttingDown = false;

  const writeSupervisor = (status: string): void => {
    const temporary = `${supervisorManifest}.tmp`;
    writeFileSync(
      temporary,
      `${JSON.stringify({
        version: 1,
        status,
        launcher_pid: process.pid,
        updated_at: new Date().toISOString(),
        actors: actors.map((actor) => ({
          index: actor.index,
          gym_url: actor.gymUrl,
          manifest: actor.context.manifestPath,
          game_pid: actor.gameIdentity.processId,
          service_pid: actor.serviceIdentity.processId,
        })),
      }, null, 2)}\n`,
      "utf8",
    );
    renameSync(temporary, supervisorManifest);
  };

  const cleanup = async (reason: string): Promise<void> => {
    if (shuttingDown) return;
    shuttingDown = true;
    writeSupervisor(`stopping:${reason}`);
    await Promise.allSettled(
      actors.flatMap((actor) => [
        cleanupOwned("collector", actor.service, actor.serviceIdentity),
        cleanupOwned("Celeste", actor.game, actor.gameIdentity),
      ]),
    );
    for (const actor of actors) {
      updateRunManifest(actor.context, { status: "persistent-actor-stopped", stop_reason: reason });
    }
    writeSupervisor("stopped");
  };

  process.once("SIGINT", () => void cleanup("SIGINT"));
  process.once("SIGTERM", () => void cleanup("SIGTERM"));

  try {
    writeSupervisor("starting");
    for (let index = 0; index < options.count; index++) {
      actors.push(await startActor(index, options, paths, gameInstall, git));
      writeSupervisor("starting");
    }
    writeSupervisor("ready");
    const summary = {
      status: "ready",
      manifest: supervisorManifest,
      actors: actors.map((actor) => ({
        index: actor.index,
        gym_url: actor.gymUrl,
        manifest: actor.context.manifestPath,
        game_pid: actor.gameIdentity.processId,
        service_pid: actor.serviceIdentity.processId,
      })),
    };
    process.stdout.write(`${JSON.stringify(summary, null, 2)}\n`);

    if (options.smoke) {
      await cleanup("smoke-complete");
      return;
    }

    await new Promise<void>((resolveWait, rejectWait) => {
      for (const actor of actors) {
        actor.game.once("exit", (code, signal) => {
          if (!shuttingDown) rejectWait(new Error(
            `Celeste actor ${actor.index} exited unexpectedly: code=${code} signal=${signal}`,
          ));
        });
        actor.service.once("exit", (code, signal) => {
          if (!shuttingDown) rejectWait(new Error(
            `collector actor ${actor.index} exited unexpectedly: code=${code} signal=${signal}`,
          ));
        });
      }
      const finish = (): void => {
        if (shuttingDown) resolveWait();
        else setTimeout(finish, 100).unref();
      };
      finish();
    });
  } catch (error) {
    await cleanup("error");
    throw error;
  }
}

async function startActor(
  index: number,
  options: ActorOptions,
  paths: ReturnType<typeof createHarnessPaths>,
  gameInstall: ReturnType<typeof validateGameInstall>,
  git: GitIdentity,
): Promise<RunningActor> {
  const modPort = await reserveLoopbackPort();
  const httpPort = await reserveLoopbackPort();
  const context = createRunContext({
    repoRoot,
    gameInstall,
    modPort: modPort.port,
    httpPort: httpPort.port,
    git,
  });
  let game: ChildProcess | undefined;
  let service: ChildProcess | undefined;
  let gameIdentity: ProcessIdentity | undefined;
  let serviceIdentity: ProcessIdentity | undefined;
  try {
    await modPort.release();
    updateRunManifest(context, { status: "persistent-actor-starting-game", actor_index: index });
    game = spawn(gameInstall.executable, ["--disable-splash", "--loglevel", "info"], {
      cwd: gameInstall.gameRoot,
      windowsHide: !options.showWindows,
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
    if (!game.pid) throw new Error(`Celeste actor ${index} exposed no process id`);
    gameIdentity = await waitForProcessIdentity(game.pid, gameInstall.executable);
    updateRunManifest(context, { status: "persistent-actor-waiting-for-everest", game_process: gameIdentity });
    const ping = await waitForOwnedEverest(
      modPort.port,
      { runNonce: context.runNonce, processId: game.pid, port: modPort.port },
      EVEREST_READY_TIMEOUT_MS,
    );
    updateRunManifest(context, { status: "persistent-actor-game-authenticated", everest_ping: ping });

    await httpPort.release();
    service = spawn(process.execPath, [resolve(paths.serviceRoot, "dist", "src", "index.js")], {
      cwd: paths.serviceRoot,
      windowsHide: true,
      stdio: "ignore",
      shell: false,
      env: {
        ...process.env,
        COLLECTOR_BACKEND: "everest",
        COLLECTOR_PORT: String(httpPort.port),
        COLLECTOR_TIMEOUT_MS: "60000",
        EVEREST_COLLECTOR_PORT: String(modPort.port),
        EVEREST_RUN_NONCE: context.runNonce,
        EVEREST_PROCESS_ID: String(gameIdentity.processId),
        EVEREST_AREA_ID: String(options.areaId),
        ...(options.areaSid ? { EVEREST_AREA_SID: options.areaSid } : {}),
      },
    });
    if (!service.pid) throw new Error(`collector actor ${index} exposed no process id`);
    serviceIdentity = await waitForProcessIdentity(service.pid, process.execPath);
    await waitForPort(httpPort.port, SERVICE_PORT_TIMEOUT_MS);
    const health = await createCollectorClient(paths.serviceRoot, httpPort.port).waitUntilReady();
    const gymUrl = `http://127.0.0.1:${httpPort.port}/api/gym`;
    updateRunManifest(context, {
      status: "persistent-actor-ready",
      service_process: serviceIdentity,
      gym_url: gymUrl,
      health,
    });
    return {
      index,
      context,
      gymUrl,
      game,
      gameIdentity,
      service,
      serviceIdentity,
    };
  } catch (error) {
    await Promise.allSettled([
      gameIdentity ? cleanupOwned("Celeste", game, gameIdentity) : Promise.resolve(false),
      serviceIdentity ? cleanupOwned("collector", service, serviceIdentity) : Promise.resolve(false),
    ]);
    updateRunManifest(context, { status: "persistent-actor-failed", error: String(error) });
    throw error;
  } finally {
    await Promise.allSettled([modPort.release(), httpPort.release()]);
  }
}

async function cleanupOwned(
  label: string,
  child: ChildProcess | undefined,
  identity: ProcessIdentity,
): Promise<boolean> {
  try {
    return await terminateOwnedProcess({ child, expectedIdentity: identity });
  } catch (error) {
    console.warn(`failed to terminate owned ${label} PID ${identity.processId}: ${String(error)}`);
    return false;
  }
}

function requiredValue(value: string | undefined, flag: string): string {
  if (!value || value.startsWith("--")) throw new Error(`${flag} requires a value`);
  return value;
}

function boundedInteger(
  value: string | undefined,
  flag: string,
  minimum: number,
  maximum: number,
): number {
  const raw = requiredValue(value, flag);
  const parsed = Number.parseInt(raw, 10);
  if (!Number.isSafeInteger(parsed) || String(parsed) !== raw || parsed < minimum || parsed > maximum) {
    throw new Error(`${flag} must be an integer from ${minimum} through ${maximum}`);
  }
  return parsed;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  void runActorLauncher().catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });
}
