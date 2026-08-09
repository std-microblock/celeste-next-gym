import { randomUUID } from "node:crypto";
import { spawn, type ChildProcess } from "node:child_process";
import { closeSync, mkdirSync, openSync, renameSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "node:net";

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
  readonly soakResets: number;
  readonly soakRoom: string;
  readonly soakFrames: number;
  readonly soakRestartAt: number;
}

interface RunningActor {
  readonly index: number;
  readonly context: RunContext;
  readonly gymUrl: string;
  readonly game: ChildProcess;
  readonly gameIdentity: ProcessIdentity;
  readonly service: ChildProcess;
  readonly serviceIdentity: ProcessIdentity;
  readonly gameStdoutPath: string;
  readonly gameStderrPath: string;
  readonly serviceStdoutPath: string;
  readonly serviceStderrPath: string;
  readonly httpPort: number;
  readonly modPort: number;
  readonly generation: number;
}

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");

export function parseActorOptions(argv: readonly string[]): ActorOptions {
  let count = 1;
  let areaId = 1;
  let areaSid: string | undefined;
  let showWindows = false;
  let smoke = false;
  let soakResets = 0;
  let soakRoom = "2";
  let soakFrames = 1536;
  let soakRestartAt = 0;
  for (let index = 0; index < argv.length; index++) {
    const argument = argv[index];
    if (argument === "--actors") count = boundedInteger(argv[++index], argument, 1, 32);
    else if (argument === "--area-id") areaId = boundedInteger(argv[++index], argument, 0, 10_000);
    else if (argument === "--area-sid") areaSid = requiredValue(argv[++index], argument);
    else if (argument === "--show-windows") showWindows = true;
    else if (argument === "--smoke") smoke = true;
    else if (argument === "--soak-resets") soakResets = boundedInteger(argv[++index], argument, 1, 1_000_000);
    else if (argument === "--soak-room") soakRoom = requiredValue(argv[++index], argument);
    else if (argument === "--soak-frames") soakFrames = boundedInteger(argv[++index], argument, 1, 4096);
    else if (argument === "--soak-restart-at") soakRestartAt = boundedInteger(argv[++index], argument, 1, 1_000_000);
    else throw new Error(`unknown gym actor argument: ${argument}`);
  }
  return Object.freeze({
    count,
    areaId,
    ...(areaSid ? { areaSid } : {}),
    showWindows,
    smoke,
    soakResets,
    soakRoom,
    soakFrames,
    soakRestartAt,
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
  const restartCounts = new Map<number, number>();
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
          restart_count: restartCounts.get(actor.index) ?? 0,
          generation: actor.generation,
          logs: {
            game_stdout: actor.gameStdoutPath,
            game_stderr: actor.gameStderrPath,
            service_stdout: actor.serviceStdoutPath,
            service_stderr: actor.serviceStderrPath,
          },
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

  const restartActor = async (index: number, reason: string): Promise<RunningActor> => {
    const previous = actors[index];
    if (!previous) throw new Error(`cannot restart missing actor ${index}`);
    updateRunManifest(previous.context, {
      status: "persistent-actor-restarting",
      restart_reason: reason,
    });
    await Promise.allSettled([
      cleanupOwned("collector", previous.service, previous.serviceIdentity),
      cleanupOwned("Celeste", previous.game, previous.gameIdentity),
    ]);
    await Promise.all([
      waitForLoopbackPortFree(previous.modPort),
      waitForLoopbackPortFree(previous.httpPort),
    ]);
    const replacement = await startActor(
      index,
      options,
      paths,
      gameInstall,
      git,
      {
        modPort: previous.modPort,
        httpPort: previous.httpPort,
        generation: previous.generation + 1,
      },
    );
    actors[index] = replacement;
    restartCounts.set(index, (restartCounts.get(index) ?? 0) + 1);
    writeSupervisor("ready");
    process.stdout.write(`${JSON.stringify({
      actor_restarted: index,
      reason,
      restart_count: restartCounts.get(index),
      generation: replacement.generation,
      gym_url: replacement.gymUrl,
      manifest: replacement.context.manifestPath,
    })}\n`);
    return replacement;
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
        logs: {
          game_stdout: actor.gameStdoutPath,
          game_stderr: actor.gameStderrPath,
          service_stdout: actor.serviceStdoutPath,
          service_stderr: actor.serviceStderrPath,
        },
      })),
    };
    process.stdout.write(`${JSON.stringify(summary, null, 2)}\n`);

    if (options.smoke) {
      await cleanup("smoke-complete");
      return;
    }
    if (options.soakResets > 0) {
      await runSoak(actors, paths.serviceRoot, options, restartActor);
      await cleanup("soak-complete");
      return;
    }

    while (!shuttingDown) {
      for (let index = 0; index < actors.length && !shuttingDown; index++) {
        const actor = actors[index]!;
        if (actor.game.exitCode !== null || actor.service.exitCode !== null) {
          await restartActor(
            index,
            `child-exit:game=${actor.game.exitCode}:service=${actor.service.exitCode}`,
          );
        }
      }
      await new Promise<void>((resolveWait) => setTimeout(resolveWait, 250));
    }
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
  fixed?: {
    readonly modPort: number;
    readonly httpPort: number;
    readonly generation: number;
  },
): Promise<RunningActor> {
  const modPort = fixed
    ? { port: fixed.modPort, release: async () => {} }
    : await reserveLoopbackPort();
  const httpPort = fixed
    ? { port: fixed.httpPort, release: async () => {} }
    : await reserveLoopbackPort();
  const generation = fixed?.generation ?? 0;
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
  const logRoot = resolve(context.runRoot, "logs");
  mkdirSync(logRoot, { recursive: true });
  const gameStdoutPath = resolve(logRoot, "celeste.stdout.log");
  const gameStderrPath = resolve(logRoot, "celeste.stderr.log");
  const serviceStdoutPath = resolve(logRoot, "collector.stdout.log");
  const serviceStderrPath = resolve(logRoot, "collector.stderr.log");
  try {
    await modPort.release();
    updateRunManifest(context, {
      status: "persistent-actor-starting-game",
      actor_index: index,
      generation,
    });
    const gameStdoutFd = openSync(gameStdoutPath, "a");
    const gameStderrFd = openSync(gameStderrPath, "a");
    try {
      game = spawn(gameInstall.executable, ["--disable-splash", "--loglevel", "info"], {
        cwd: gameInstall.gameRoot,
        windowsHide: !options.showWindows,
        stdio: ["ignore", gameStdoutFd, gameStderrFd],
        shell: false,
        env: {
          ...process.env,
          CELESTE_GYM_COLLECTOR_PORT: String(modPort.port),
          CELESTE_GYM_RUN_NONCE: context.runNonce,
          EVEREST_SAVEPATH: context.saveRoot,
          EVEREST_TMPDIR: context.tempRoot,
        },
      });
    } finally {
      closeSync(gameStdoutFd);
      closeSync(gameStderrFd);
    }
    if (!game.pid) throw new Error(`Celeste actor ${index} exposed no process id`);
    game.once("exit", (code, signal) => {
      updateRunManifest(context, {
        game_exit: { code, signal, at: new Date().toISOString() },
      });
    });
    gameIdentity = await waitForProcessIdentity(game.pid, gameInstall.executable);
    updateRunManifest(context, { status: "persistent-actor-waiting-for-everest", game_process: gameIdentity });
    const ping = await waitForOwnedEverest(
      modPort.port,
      { runNonce: context.runNonce, processId: game.pid, port: modPort.port },
      EVEREST_READY_TIMEOUT_MS,
    );
    updateRunManifest(context, { status: "persistent-actor-game-authenticated", everest_ping: ping });

    await httpPort.release();
    const serviceStdoutFd = openSync(serviceStdoutPath, "a");
    const serviceStderrFd = openSync(serviceStderrPath, "a");
    try {
      service = spawn(process.execPath, [resolve(paths.serviceRoot, "dist", "src", "index.js")], {
        cwd: paths.serviceRoot,
        windowsHide: true,
        stdio: ["ignore", serviceStdoutFd, serviceStderrFd],
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
    } finally {
      closeSync(serviceStdoutFd);
      closeSync(serviceStderrFd);
    }
    if (!service.pid) throw new Error(`collector actor ${index} exposed no process id`);
    service.once("exit", (code, signal) => {
      updateRunManifest(context, {
        service_exit: { code, signal, at: new Date().toISOString() },
      });
    });
    serviceIdentity = await waitForProcessIdentity(service.pid, process.execPath);
    await waitForPort(httpPort.port, SERVICE_PORT_TIMEOUT_MS);
    const health = await createCollectorClient(paths.serviceRoot, httpPort.port).waitUntilReady();
    const gymUrl = `http://127.0.0.1:${httpPort.port}/api/gym`;
    updateRunManifest(context, {
      status: "persistent-actor-ready",
      service_process: serviceIdentity,
      gym_url: gymUrl,
      health,
      logs: {
        game_stdout: gameStdoutPath,
        game_stderr: gameStderrPath,
        service_stdout: serviceStdoutPath,
        service_stderr: serviceStderrPath,
      },
    });
    return {
      index,
      context,
      gymUrl,
      game,
      gameIdentity,
      service,
      serviceIdentity,
      gameStdoutPath,
      gameStderrPath,
      serviceStdoutPath,
      serviceStderrPath,
      httpPort: httpPort.port,
      modPort: modPort.port,
      generation,
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

async function waitForLoopbackPortFree(port: number, timeoutMs = 10_000): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  do {
    const available = await new Promise<boolean>((resolveAvailable) => {
      const server = createServer();
      server.unref();
      server.once("error", () => resolveAvailable(false));
      server.listen({ host: "127.0.0.1", port, exclusive: true }, () => {
        server.close(() => resolveAvailable(true));
      });
    });
    if (available) return;
    await new Promise<void>((resolveWait) => setTimeout(resolveWait, 50));
  } while (Date.now() < deadline);
  throw new Error(`loopback port ${port} was not released within ${timeoutMs} ms`);
}

async function runSoak(
  actors: RunningActor[],
  serviceRoot: string,
  options: ActorOptions,
  restartActor: (index: number, reason: string) => Promise<RunningActor>,
): Promise<void> {
  const neutral = Array.from({ length: options.soakFrames }, () => ({
    move_x: 0,
    move_y: 0,
    jump_pressed: false,
    jump_held: false,
    dash_pressed: false,
    crouch_dash_pressed: false,
    grab_held: false,
    talk_pressed: false,
  }));
  const started = performance.now();
  let physicsFrames = 0;
  let latestEntityCount = 0;
  let maximumEntityCount = 0;
  for (let resetIndex = 0; resetIndex < options.soakResets; resetIndex++) {
    if (options.soakRestartAt === resetIndex + 1) {
      for (let actorIndex = 0; actorIndex < actors.length; actorIndex++) {
        const previousUrl = actors[actorIndex]!.gymUrl;
        const replacement = await restartActor(actorIndex, "forced-soak-restart");
        if (replacement.gymUrl !== previousUrl) {
          throw new Error(
            `actor ${actorIndex} public Gym URL changed across restart: ` +
            `${previousUrl} -> ${replacement.gymUrl}`
          );
        }
      }
    }
    await Promise.all(actors.map(async (_, actorIndex) => {
      for (let attempt = 0; ; attempt++) {
        const actor = actors[actorIndex]!;
        try {
          const client = createCollectorClient(serviceRoot, actor.httpPort);
          const reset = await client.gymReset({
            area_id: options.areaId,
            ...(options.areaSid ? { area_sid: options.areaSid } : {}),
            room: options.soakRoom,
            skip_transitions: true,
            max_episode_frames: options.soakFrames,
            include_entities: true,
            include_player_states: false,
            fast_mode: true,
          });
          const episodeId = reset.observation?.episode_id;
          if (typeof episodeId !== "string") throw new Error("soak reset returned no episode id");
          const entityCount = Array.isArray(reset.observation?.entities)
            ? reset.observation.entities.length
            : 0;
          latestEntityCount = entityCount;
          maximumEntityCount = Math.max(maximumEntityCount, entityCount);
          const step = await client.gymStep({ episode_id: episodeId, inputs: neutral });
          physicsFrames += step.frames_executed;
          return;
        } catch (error) {
          if (attempt >= 3) throw error;
          await restartActor(actorIndex, `soak-backend-error:${String(error)}`);
        }
      }
    }));
    if ((resetIndex + 1) % 25 === 0 || resetIndex + 1 === options.soakResets) {
      const elapsedSeconds = (performance.now() - started) / 1000;
      process.stdout.write(
        `${JSON.stringify({
          soak_resets: resetIndex + 1,
          actors: actors.length,
          physics_frames: physicsFrames,
          elapsed_seconds: Number(elapsedSeconds.toFixed(1)),
          effective_fps: Number((physicsFrames / Math.max(elapsedSeconds, 0.001)).toFixed(1)),
          entity_count: latestEntityCount,
          maximum_entity_count: maximumEntityCount,
        })}\n`,
      );
    }
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
