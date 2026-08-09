import { randomUUID } from "node:crypto";
import { spawn, type ChildProcess } from "node:child_process";
import { closeSync, mkdirSync, openSync, renameSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createConnection, createServer, type Socket } from "node:net";
import { isDeepStrictEqual } from "node:util";

import {
  createRunContext,
  reserveLongLivedLoopbackPort,
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
  readonly seedSmoke: boolean;
  readonly seedSmokeSeed: number;
  readonly soakResets: number;
  readonly soakRoom: string;
  readonly soakFrames: number;
  readonly soakRestartAt: number;
  readonly soakPolicy: boolean;
  readonly soakSeed: number;
  readonly soakActionFrames: number;
  readonly directTcp: boolean;
}

interface SoakInput {
  readonly move_x: number;
  readonly move_y: number;
  readonly jump_pressed: boolean;
  readonly jump_held: boolean;
  readonly dash_pressed: boolean;
  readonly crouch_dash_pressed: boolean;
  readonly grab_held: boolean;
  readonly talk_pressed: boolean;
}

interface ActorGymResult {
  readonly observation?: Record<string, unknown>;
  readonly player_states: readonly Record<string, unknown>[];
  readonly frames_executed: number;
}

interface ActorGymClient {
  gymReset(request: Record<string, unknown>): Promise<ActorGymResult>;
  gymStep(request: Record<string, unknown>): Promise<ActorGymResult>;
}

interface RunningActor {
  readonly index: number;
  readonly context: RunContext;
  readonly gymUrl?: string;
  readonly game: ChildProcess;
  readonly gameIdentity: ProcessIdentity;
  readonly service?: ChildProcess;
  readonly serviceIdentity?: ProcessIdentity;
  readonly gameStdoutPath: string;
  readonly gameStderrPath: string;
  readonly everestLogPath: string;
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
  let seedSmoke = false;
  let seedSmokeSeed = 8_675_309;
  let soakResets = 0;
  let soakRoom = "2";
  let soakFrames = 1536;
  let soakRestartAt = 0;
  let soakPolicy = false;
  let soakSeed = 1;
  let soakActionFrames = 8;
  let directTcp = false;
  for (let index = 0; index < argv.length; index++) {
    const argument = argv[index];
    if (argument === "--actors") count = boundedInteger(argv[++index], argument, 1, 32);
    else if (argument === "--area-id") areaId = boundedInteger(argv[++index], argument, 0, 10_000);
    else if (argument === "--area-sid") areaSid = requiredValue(argv[++index], argument);
    else if (argument === "--show-windows") showWindows = true;
    else if (argument === "--smoke") smoke = true;
    else if (argument === "--seed-smoke") seedSmoke = true;
    else if (argument === "--seed-smoke-seed") seedSmokeSeed = boundedInteger(
      argv[++index],
      argument,
      -0x8000_0000,
      0x7fff_ffff,
    );
    else if (argument === "--soak-resets") soakResets = boundedInteger(argv[++index], argument, 1, 1_000_000);
    else if (argument === "--soak-room") soakRoom = requiredValue(argv[++index], argument);
    else if (argument === "--soak-frames") soakFrames = boundedInteger(argv[++index], argument, 1, 4096);
    else if (argument === "--soak-restart-at") soakRestartAt = boundedInteger(argv[++index], argument, 1, 1_000_000);
    else if (argument === "--soak-policy") soakPolicy = true;
    else if (argument === "--soak-seed") soakSeed = boundedInteger(argv[++index], argument, 0, 0xffff_ffff);
    else if (argument === "--soak-action-frames") soakActionFrames = boundedInteger(argv[++index], argument, 1, 64);
    else if (argument === "--direct-tcp") directTcp = true;
    else throw new Error(`unknown gym actor argument: ${argument}`);
  }
  return Object.freeze({
    count,
    areaId,
    ...(areaSid ? { areaSid } : {}),
    showWindows,
    smoke,
    seedSmoke,
    seedSmokeSeed,
    soakResets,
    soakRoom,
    soakFrames,
    soakRestartAt,
    soakPolicy,
    soakSeed,
    soakActionFrames,
    directTcp,
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
          tcp_endpoint: `tcp://127.0.0.1:${actor.modPort}`,
          tcp_host: "127.0.0.1",
          tcp_port: actor.modPort,
          auth: {
            run_nonce: actor.context.runNonce,
            process_id: actor.gameIdentity.processId,
          },
          ...(actor.gymUrl ? { gym_url: actor.gymUrl } : {}),
          manifest: actor.context.manifestPath,
          game_pid: actor.gameIdentity.processId,
          ...(actor.serviceIdentity ? { service_pid: actor.serviceIdentity.processId } : {}),
          restart_count: restartCounts.get(actor.index) ?? 0,
          generation: actor.generation,
          logs: {
            game_stdout: actor.gameStdoutPath,
            game_stderr: actor.gameStderrPath,
            everest_log: actor.everestLogPath,
            ...(actor.serviceIdentity ? {
              service_stdout: actor.serviceStdoutPath,
              service_stderr: actor.serviceStderrPath,
            } : {}),
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
        ...(actor.serviceIdentity
          ? [cleanupOwned("collector", actor.service, actor.serviceIdentity)]
          : []),
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
      ...(previous.serviceIdentity
        ? [cleanupOwned("collector", previous.service, previous.serviceIdentity)]
        : []),
      cleanupOwned("Celeste", previous.game, previous.gameIdentity),
    ]);
    await Promise.all([
      waitForLoopbackPortFree(previous.modPort),
      ...(previous.httpPort ? [waitForLoopbackPortFree(previous.httpPort)] : []),
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
      tcp_endpoint: `tcp://127.0.0.1:${replacement.modPort}`,
      auth: {
        run_nonce: replacement.context.runNonce,
        process_id: replacement.gameIdentity.processId,
      },
      ...(replacement.gymUrl ? { gym_url: replacement.gymUrl } : {}),
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
        tcp_endpoint: `tcp://127.0.0.1:${actor.modPort}`,
        tcp_host: "127.0.0.1",
        tcp_port: actor.modPort,
        auth: {
          run_nonce: actor.context.runNonce,
          process_id: actor.gameIdentity.processId,
        },
        ...(actor.gymUrl ? { gym_url: actor.gymUrl } : {}),
        manifest: actor.context.manifestPath,
        game_pid: actor.gameIdentity.processId,
        ...(actor.serviceIdentity ? { service_pid: actor.serviceIdentity.processId } : {}),
        logs: {
          game_stdout: actor.gameStdoutPath,
          game_stderr: actor.gameStderrPath,
          everest_log: actor.everestLogPath,
          ...(actor.serviceIdentity ? {
            service_stdout: actor.serviceStdoutPath,
            service_stderr: actor.serviceStderrPath,
          } : {}),
        },
      })),
    };
    process.stdout.write(`${JSON.stringify(summary, null, 2)}\n`);

    if (options.seedSmoke) {
      await runSeedTrajectoryGate(actors, paths.serviceRoot, options);
      await cleanup("seed-smoke-complete");
      return;
    }
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
        if (actor.game.exitCode !== null || actor.service?.exitCode !== null && actor.service?.exitCode !== undefined) {
          await restartActor(
            index,
            `child-exit:game=${actor.game.exitCode}:service=${actor.service?.exitCode ?? "none"}`,
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

async function runSeedTrajectoryGate(
  actors: readonly RunningActor[],
  serviceRoot: string,
  options: ActorOptions,
): Promise<void> {
  const inputs = createSeedTrajectoryInputs();
  const started = performance.now();
  const results = await Promise.all(actors.map(async (actor) => {
    const client = createActorGymClient(actor, serviceRoot);
    const perturbSeed = options.seedSmokeSeed === 0x7fff_ffff
      ? -0x8000_0000
      : options.seedSmokeSeed + 1;

    // Put the actor on the requested map first so both compared resets exercise
    // the long-lived in-place Reload path used by training after actor startup.
    await captureSeededTrajectory(client, options, perturbSeed, inputs.slice().reverse());
    const first = await captureSeededTrajectory(
      client,
      options,
      options.seedSmokeSeed,
      inputs,
    );

    // Deliberately replace and consume the authoritative stream between the
    // two target runs. The second reset must still reproduce the first run.
    await captureSeededTrajectory(client, options, perturbSeed, inputs.slice().reverse());

    const second = await captureSeededTrajectory(
      client,
      options,
      options.seedSmokeSeed,
      inputs,
    );
    const firstComparable = canonicalizeSeedTrajectory(first);
    const secondComparable = canonicalizeSeedTrajectory(second);
    if (!isDeepStrictEqual(firstComparable, secondComparable)) {
      throw new Error(
        `actor ${actor.index} same-seed trajectory mismatch: ` +
        `${describeTrajectoryMismatch(
          firstComparable.playerStates,
          secondComparable.playerStates,
        )}`,
      );
    }
    return {
      actor: actor.index,
      seed: options.seedSmokeSeed,
      frames: first.playerStates.length - 1,
      terminal: first.terminal,
      final_player: canonicalizePlayerState(first.finalPlayer),
    };
  }));
  process.stdout.write(`${JSON.stringify({
    seed_trajectory_gate: "exact-pass",
    room: options.soakRoom,
    input_frames: inputs.length,
    actors: results,
    elapsed_seconds: Number(((performance.now() - started) / 1000).toFixed(2)),
  })}\n`);
}

function canonicalizeSeedTrajectory(
  trajectory: CapturedSeedTrajectory,
): CapturedSeedTrajectory {
  return {
    playerStates: trajectory.playerStates.map((state) =>
      canonicalizePlayerState(state) as Record<string, unknown>),
    finalPlayer: canonicalizePlayerState(trajectory.finalPlayer),
    terminal: trajectory.terminal,
  };
}

function canonicalizePlayerState(value: unknown): unknown {
  if (!value || typeof value !== "object" || Array.isArray(value)) return value;
  const state = value as Record<string, unknown>;
  // Reflected internals include presentation-only values driven by continuous
  // Scene.Time (for example Player.flash) and Monocle's entity insertion epsilon
  // (actualDepth). The authoritative trajectory is the complete top-level
  // PlayerFrame: position, speed, state, facing, resources, contacts and death.
  const {
    fields: _directReflectedInternals,
    _everest_fields: _httpReflectedInternals,
    ...trajectoryState
  } = state;
  return trajectoryState;
}

interface CapturedSeedTrajectory {
  readonly playerStates: readonly Record<string, unknown>[];
  readonly finalPlayer: unknown;
  readonly terminal: boolean;
}

async function captureSeededTrajectory(
  client: ActorGymClient,
  options: ActorOptions,
  seed: number,
  inputs: readonly SoakInput[],
): Promise<CapturedSeedTrajectory> {
  const reset = await client.gymReset({
    area_id: options.areaId,
    ...(options.areaSid ? { area_sid: options.areaSid } : {}),
    room: options.soakRoom,
    seed,
    skip_transitions: true,
    max_episode_frames: inputs.length + 60,
    include_entities: true,
    include_player_states: true,
    fast_mode: true,
  });
  const episodeId = reset.observation?.episode_id;
  if (typeof episodeId !== "string") {
    throw new Error("seed trajectory reset returned no episode id");
  }
  const playerStates = [...reset.player_states];
  let finalPlayer = reset.observation?.player;
  let terminal = false;
  const step = await client.gymStep({ episode_id: episodeId, inputs });
  playerStates.push(...step.player_states);
  finalPlayer = step.observation?.player;
  terminal = step.observation?.terminated === true || step.observation?.truncated === true;
  if (step.frames_executed === 0 && !terminal) {
    throw new Error("seed trajectory step executed zero frames before episode completion");
  }
  return { playerStates, finalPlayer, terminal };
}

export function createSeedTrajectoryInputs(): SoakInput[] {
  return Array.from({ length: 128 }, (_, frame): SoakInput => {
    const jumpStart = frame === 16 || frame === 72;
    const jumpHeld = (frame >= 16 && frame < 28) || (frame >= 72 && frame < 84);
    const dashStart = frame === 40 || frame === 96;
    return {
      move_x: frame < 64 ? 1 : -1,
      move_y: frame >= 36 && frame < 52 ? -1 : frame >= 92 && frame < 108 ? 1 : 0,
      jump_pressed: jumpStart,
      jump_held: jumpHeld,
      dash_pressed: dashStart,
      crouch_dash_pressed: frame === 112,
      grab_held: frame >= 56 && frame < 68,
      talk_pressed: false,
    };
  });
}

function describeTrajectoryMismatch(
  first: readonly Record<string, unknown>[],
  second: readonly Record<string, unknown>[],
): string {
  if (first.length !== second.length) return `length ${first.length} != ${second.length}`;
  const index = first.findIndex((state, stateIndex) =>
    !isDeepStrictEqual(state, second[stateIndex]));
  if (index < 0) return "final observation differs";
  return `first different player state at frame index ${index}: ` +
    `${JSON.stringify(first[index])} != ${JSON.stringify(second[index])}`;
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
    : await reserveLongLivedLoopbackPort();
  const httpPort = fixed
    ? { port: fixed.httpPort, release: async () => {} }
    : await reserveLongLivedLoopbackPort();
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
  const everestLogFilename = `celeste-gym-${context.runNonce}.txt`;
  const everestLogPath = resolve(gameInstall.gameRoot, everestLogFilename);
  const serviceStdoutPath = resolve(logRoot, "collector.stdout.log");
  const serviceStderrPath = resolve(logRoot, "collector.stderr.log");
  if (!options.showWindows) {
    const savesRoot = resolve(context.saveRoot, "Saves");
    mkdirSync(savesRoot, { recursive: true });
    writeFileSync(
      resolve(savesRoot, "modsettings-Everest.celeste"),
      "DiscordRichPresence: false\nAutoUpdateModsOnStartup: false\n",
      "utf8",
    );
  }
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
          CELESTE_GYM_HEADLESS: options.showWindows ? "0" : "1",
          EVEREST_LOG_FILENAME: everestLogFilename,
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

    if (options.directTcp) {
      updateRunManifest(context, {
        status: "persistent-actor-ready",
        transport: "direct-tcp",
        tcp_endpoint: `tcp://127.0.0.1:${modPort.port}`,
        tcp_host: "127.0.0.1",
        tcp_port: modPort.port,
        auth: {
          run_nonce: context.runNonce,
          process_id: gameIdentity.processId,
        },
        logs: {
          game_stdout: gameStdoutPath,
          game_stderr: gameStderrPath,
          everest_log: everestLogPath,
        },
      });
      return {
        index,
        context,
        game,
        gameIdentity,
        gameStdoutPath,
        gameStderrPath,
        everestLogPath,
        serviceStdoutPath,
        serviceStderrPath,
        httpPort: httpPort.port,
        modPort: modPort.port,
        generation,
      };
    }

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
        everest_log: everestLogPath,
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
      everestLogPath,
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
  let episodes = 0;
  let actionBatches = 0;
  let deaths = 0;
  let roomTransitions = 0;
  let truncations = 0;
  // Use the same stream in every actor to stress synchronized policy actions,
  // which is common before vectorized environments diverge.
  const randoms = actors.map(() => createSeededRandom(options.soakSeed));
  if (options.soakPolicy && options.soakRestartAt > 0) {
    throw new Error("--soak-policy cannot be combined with --soak-restart-at");
  }
  for (let resetIndex = 0; resetIndex < options.soakResets; resetIndex++) {
    if (options.soakRestartAt === resetIndex + 1) {
      for (let actorIndex = 0; actorIndex < actors.length; actorIndex++) {
        const previousUrl = actors[actorIndex]!.gymUrl;
        const previousPort = actors[actorIndex]!.modPort;
        const previousNonce = actors[actorIndex]!.context.runNonce;
        const previousProcessId = actors[actorIndex]!.gameIdentity.processId;
        const replacement = await restartActor(actorIndex, "forced-soak-restart");
        if (replacement.modPort !== previousPort) {
          throw new Error(
            `actor ${actorIndex} public Mod TCP port changed across restart: ` +
            `${previousPort} -> ${replacement.modPort}`
          );
        }
        if (replacement.context.runNonce === previousNonce
            || replacement.gameIdentity.processId === previousProcessId) {
          throw new Error(
            `actor ${actorIndex} did not rotate TCP generation ownership across restart`
          );
        }
        if (!options.directTcp && replacement.gymUrl !== previousUrl) {
          throw new Error(
            `actor ${actorIndex} public Gym URL changed across restart: ` +
            `${previousUrl} -> ${replacement.gymUrl}`
          );
        }
      }
    }
    await Promise.all(actors.map(async (_, actorIndex) => {
      if (options.soakPolicy) {
        const actor = actors[actorIndex]!;
        const client = createActorGymClient(actor, serviceRoot);
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
        if (typeof episodeId !== "string") {
          throw new Error("policy soak reset returned no episode id");
        }
        const entityCount = Array.isArray(reset.observation?.entities)
          ? reset.observation.entities.length
          : 0;
        latestEntityCount = entityCount;
        maximumEntityCount = Math.max(maximumEntityCount, entityCount);
        episodes++;
        let episodeFrames = 0;
        let batchIndex = 0;
        while (episodeFrames < options.soakFrames) {
          if (actor.game.exitCode !== null || (actor.service?.exitCode ?? null) !== null) {
            throw new Error(
              `actor ${actorIndex} exited during strict policy soak: ` +
              `game=${actor.game.exitCode}:service=${actor.service.exitCode}`
            );
          }
          const inputs = createPolicySoakBatch(
            randoms[actorIndex]!,
            Math.min(options.soakActionFrames, options.soakFrames - episodeFrames),
            batchIndex++,
          );
          let step;
          try {
            step = await client.gymStep({ episode_id: episodeId, inputs });
          } catch (error) {
            throw new Error(
              `policy soak failed at episode=${resetIndex + 1} ` +
              `batch=${batchIndex} frame=${episodeFrames} ` +
              `inputs=${JSON.stringify(inputs)}: ${String(error)}`
            );
          }
          actionBatches++;
          episodeFrames += step.frames_executed;
          physicsFrames += step.frames_executed;
          const observation = step.observation;
          if (observation?.terminated === true || observation?.truncated === true) {
            const reason = observation.termination_reason;
            if (reason === "death") deaths++;
            else if (reason === "room_transition") roomTransitions++;
            else if (reason === "max_episode_frames") truncations++;
            break;
          }
          if (step.frames_executed === 0) {
            throw new Error("policy soak step executed zero frames before episode completion");
          }
        }
        return;
      }
      for (let attempt = 0; ; attempt++) {
        const actor = actors[actorIndex]!;
        try {
          const client = createActorGymClient(actor, serviceRoot);
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
          ...(options.soakPolicy ? {
            soak_mode: "policy",
            soak_seed: options.soakSeed,
            episodes,
            action_batches: actionBatches,
            deaths,
            room_transitions: roomTransitions,
            truncations,
          } : { soak_mode: "neutral" }),
        })}\n`,
      );
    }
  }
}

const directGymClients = new WeakMap<RunningActor, ActorGymClient>();

function createActorGymClient(actor: RunningActor, serviceRoot: string): ActorGymClient {
  if (actor.serviceIdentity) {
    return createCollectorClient(serviceRoot, actor.httpPort);
  }
  const existing = directGymClients.get(actor);
  if (existing) return existing;
  const transport = new PersistentDirectGymTransport(actor);
  const client: ActorGymClient = {
    gymReset: async (request) => await transport.send({
      command: "gym_reset",
      ...request,
    }),
    gymStep: async (request) => await transport.send({
      command: "gym_step",
      ...request,
    }),
  };
  directGymClients.set(actor, client);
  return client;
}

class PersistentDirectGymTransport {
  private socket?: Socket;
  private buffer = "";
  private connecting?: Promise<void>;
  private pending?: {
    readonly resolve: (response: Record<string, unknown>) => void;
    readonly reject: (error: unknown) => void;
    readonly timer: ReturnType<typeof setTimeout>;
  };

  public constructor(private readonly actor: RunningActor) {}

  public async send(request: Record<string, unknown>): Promise<ActorGymResult> {
    if (this.pending) throw new Error("direct Gym TCP permits one in-flight request per actor");
    await this.ensureConnected();
    const payload = {
      ...request,
      run_nonce: this.actor.context.runNonce,
      process_id: this.actor.gameIdentity.processId,
    };
    const response = await new Promise<Record<string, unknown>>((resolveResponse, reject) => {
      const timer = setTimeout(() => {
        this.failPending(new Error("direct Gym TCP request timed out after 60000 ms"));
        this.resetSocket();
      }, 60_000);
      timer.unref();
      this.pending = { resolve: resolveResponse, reject, timer };
      this.socket!.write(`${JSON.stringify(payload)}\n`);
    });
    return validateDirectGymResponse(response);
  }

  private async ensureConnected(): Promise<void> {
    if (this.socket && !this.socket.destroyed) return;
    if (this.connecting) return await this.connecting;
    this.connecting = new Promise<void>((resolveConnected, reject) => {
      const socket = createConnection({ host: "127.0.0.1", port: this.actor.modPort });
      this.socket = socket;
      socket.setEncoding("utf8");
      socket.setNoDelay(true);
      socket.once("connect", resolveConnected);
      socket.on("data", (chunk: string) => this.receive(chunk));
      socket.on("error", (error) => {
        this.failPending(error);
        if (!socket.connecting) this.resetSocket(socket);
        else reject(error);
      });
      socket.on("close", () => {
        this.failPending(new Error("direct Gym TCP connection closed"));
        this.resetSocket(socket);
      });
    }).finally(() => {
      this.connecting = undefined;
    });
    return await this.connecting;
  }

  private receive(chunk: string): void {
    this.buffer += chunk;
    const newline = this.buffer.indexOf("\n");
    if (newline < 0) return;
    const line = this.buffer.slice(0, newline);
    this.buffer = this.buffer.slice(newline + 1);
    const pending = this.pending;
    if (!pending) {
      this.resetSocket();
      return;
    }
    this.pending = undefined;
    clearTimeout(pending.timer);
    try {
      const decoded = JSON.parse(line);
      if (!decoded || typeof decoded !== "object" || Array.isArray(decoded)) {
        throw new Error("response is not an object");
      }
      pending.resolve(decoded as Record<string, unknown>);
    } catch (error) {
      pending.reject(new Error(`invalid direct Gym TCP response: ${String(error)}`));
      this.resetSocket();
    }
  }

  private failPending(error: unknown): void {
    const pending = this.pending;
    if (!pending) return;
    this.pending = undefined;
    clearTimeout(pending.timer);
    pending.reject(error);
  }

  private resetSocket(expected?: Socket): void {
    if (expected && this.socket !== expected) return;
    const socket = this.socket;
    this.socket = undefined;
    this.buffer = "";
    socket?.destroy();
  }
}

function validateDirectGymResponse(response: Record<string, unknown>): ActorGymResult {
  if (response.success !== true) {
    throw new Error(`direct Gym TCP failed: ${String(response.error ?? "unknown error")}`);
  }
  const observation = response.observation;
  if (observation !== undefined
      && (!observation || typeof observation !== "object" || Array.isArray(observation))) {
    throw new Error("direct Gym TCP observation is not an object");
  }
  const framesExecuted = response.frames_executed ?? 0;
  if (!Number.isSafeInteger(framesExecuted) || (framesExecuted as number) < 0) {
    throw new Error("direct Gym TCP frames_executed is invalid");
  }
  const playerStates = response.player_states ?? [];
  if (!Array.isArray(playerStates)) {
    throw new Error("direct Gym TCP player_states is not an array");
  }
  return {
    ...(observation === undefined
      ? {}
      : { observation: observation as Record<string, unknown> }),
    player_states: playerStates as Record<string, unknown>[],
    frames_executed: framesExecuted as number,
  };
}

export function createSeededRandom(seed: number): () => number {
  let state = seed >>> 0;
  return () => {
    state = (state + 0x6d2b79f5) >>> 0;
    let value = state;
    value = Math.imul(value ^ (value >>> 15), value | 1);
    value ^= value + Math.imul(value ^ (value >>> 7), value | 61);
    return ((value ^ (value >>> 14)) >>> 0) / 0x1_0000_0000;
  };
}

export function createPolicySoakBatch(
  random: () => number,
  maximumFrames: number,
  batchIndex: number,
): SoakInput[] {
  const forcedAction = batchIndex % 16;
  const action = forcedAction < 8 ? forcedAction : Math.floor(random() * 8);
  const duration = Math.max(1, maximumFrames);
  const horizontal = random() < 0.5 ? -1 : 1;
  const vertical = random() < 0.34 ? -1 : random() < 0.5 ? 0 : 1;
  return Array.from({ length: duration }, (_, frame): SoakInput => ({
    move_x: action === 1 ? -1 : action === 2 ? 1 : action >= 3 ? horizontal : 0,
    move_y: action >= 5 ? vertical : 0,
    jump_pressed: action === 3 && frame === 0,
    jump_held: action === 3,
    dash_pressed: action === 4 && frame === 0,
    crouch_dash_pressed: action === 5 && frame === 0,
    grab_held: action === 6,
    talk_pressed: action === 7 && frame === 0,
  }));
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
