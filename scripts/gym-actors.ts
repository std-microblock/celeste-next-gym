import { randomUUID } from "node:crypto";
import { spawn, type ChildProcess } from "node:child_process";
import { closeSync, mkdirSync, openSync, readFileSync, renameSync, writeFileSync } from "node:fs";
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
  readonly areaMode: number;
  readonly areaSid?: string;
  readonly showWindows: boolean;
  readonly smoke: boolean;
  readonly seedSmoke: boolean;
  readonly seedSmokeSeed: number;
  readonly inputLifecycleSmoke: boolean;
  readonly inputLifecycleRounds: number;
  readonly expertReplaySmoke: boolean;
  readonly expertReplayRounds: number;
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
const city2ExpertActionsPath = resolve(
  repoRoot,
  "scripts",
  "e2e-real",
  "fixtures",
  "city2-expert-actions.json",
);

export function parseActorOptions(argv: readonly string[]): ActorOptions {
  let count = 1;
  let areaId = 1;
  let areaMode = 0;
  let areaSid: string | undefined;
  let showWindows = false;
  let smoke = false;
  let seedSmoke = false;
  let seedSmokeSeed = 8_675_309;
  let inputLifecycleSmoke = false;
  let inputLifecycleRounds = 100;
  let expertReplaySmoke = false;
  let expertReplayRounds = 5;
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
    else if (argument === "--area-mode") areaMode = boundedInteger(argv[++index], argument, 0, 2);
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
    else if (argument === "--input-lifecycle-smoke") inputLifecycleSmoke = true;
    else if (argument === "--input-lifecycle-rounds") inputLifecycleRounds = boundedInteger(
      argv[++index],
      argument,
      1,
      10_000,
    );
    else if (argument === "--expert-replay-smoke") expertReplaySmoke = true;
    else if (argument === "--expert-replay-rounds") expertReplayRounds = boundedInteger(
      argv[++index],
      argument,
      2,
      1_000,
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
    areaMode,
    ...(areaSid ? { areaSid } : {}),
    showWindows,
    smoke,
    seedSmoke,
    seedSmokeSeed,
    inputLifecycleSmoke,
    inputLifecycleRounds,
    expertReplaySmoke,
    expertReplayRounds,
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

    if (options.expertReplaySmoke) {
      await runExpertReplayGate(actors, paths.serviceRoot, options);
      await cleanup("expert-replay-smoke-complete");
      return;
    }
    if (options.inputLifecycleSmoke) {
      await runInputLifecycleGate(actors, paths.serviceRoot, options);
      await cleanup("input-lifecycle-smoke-complete");
      return;
    }
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

interface ExpertReplayCapture {
  readonly decisions: number;
  readonly frames: number;
  readonly reason: unknown;
  readonly success: boolean;
  readonly playerStates: readonly Record<string, unknown>[];
  readonly finalPlayer: unknown;
}

type ExpertDecision = readonly [number, number, number, number, number];

async function runExpertReplayGate(
  actors: readonly RunningActor[],
  serviceRoot: string,
  options: ActorOptions,
): Promise<void> {
  if (options.areaSid || options.areaId !== 1 || options.areaMode !== 0 || options.soakRoom !== "2") {
    throw new Error("--expert-replay-smoke requires vanilla area_id=1 room=2");
  }
  const actions = loadCity2ExpertActions();
  const started = performance.now();
  const results = await Promise.all(actors.map(async (actor) => {
    const client = createActorGymClient(actor, serviceRoot);
    let baseline: ExpertReplayCapture | undefined;
    for (let round = 0; round < options.expertReplayRounds; round++) {
      const capture = await captureExpertReplay(client, options, actions, 31);
      if (capture.success !== true
          || capture.reason !== "room_transition"
          || capture.decisions !== 69
          || capture.frames !== 273) {
        throw new Error(
          `actor ${actor.index} expert replay round ${round} changed outcome: ` +
          JSON.stringify({
            success: capture.success,
            reason: capture.reason,
            decisions: capture.decisions,
            frames: capture.frames,
            final_player: canonicalizePlayerState(capture.finalPlayer),
          }),
        );
      }
      if (baseline) {
        const expected = canonicalizeExpertReplay(baseline);
        const actual = canonicalizeExpertReplay(capture);
        if (!isDeepStrictEqual(expected, actual)) {
          throw new Error(
            `actor ${actor.index} expert replay round ${round} drifted: ` +
            describeTrajectoryMismatch(expected.playerStates, actual.playerStates),
          );
        }
      } else {
        baseline = capture;
      }
    }
    return {
      actor: actor.index,
      seed: 31,
      rounds: options.expertReplayRounds,
      decisions: baseline!.decisions,
      physics_frames: baseline!.frames,
      termination_reason: baseline!.reason,
    };
  }));
  process.stdout.write(`${JSON.stringify({
    expert_replay_gate: "exact-pass",
    fixture: city2ExpertActionsPath,
    inter_decision_idle_ms: 20,
    actors: results,
    elapsed_seconds: Number(((performance.now() - started) / 1000).toFixed(2)),
  })}\n`);
}

async function captureExpertReplay(
  client: ActorGymClient,
  options: ActorOptions,
  actions: readonly ExpertDecision[],
  seed: number,
): Promise<ExpertReplayCapture> {
  const reset = await client.gymReset({
    area_id: 1,
    area_mode: 0,
    room: "2",
    seed,
    initial_snapshot: options.directTcp ? { state: 0 } : null,
    skip_transitions: true,
    max_episode_frames: 1536,
    include_entities: true,
    include_player_states: true,
    fast_mode: true,
  });
  const episodeId = reset.observation?.episode_id;
  if (typeof episodeId !== "string") throw new Error("expert replay reset returned no episode id");
  const playerStates = [...reset.player_states];
  let previousJump = false;
  let frames = 0;
  let finalPlayer = reset.observation?.player;
  let reason: unknown;
  let success = false;
  for (let decision = 0; decision < actions.length; decision++) {
    const action = actions[decision]!;
    const inputs = createExpertDecisionInputs(action, previousJump);
    previousJump = action[2] === 1;
    const step = await client.gymStep({ episode_id: episodeId, inputs });
    frames += step.frames_executed;
    playerStates.push(...step.player_states);
    finalPlayer = step.observation?.player;
    reason = step.observation?.termination_reason;
    success = step.observation?.success === true;
    if (step.observation?.terminated === true || step.observation?.truncated === true) {
      return {
        decisions: decision + 1,
        frames,
        reason,
        success,
        playerStates,
        finalPlayer,
      };
    }
    // The old backend advanced Level/Player here while no gym_step was active.
    // A deliberate wall-clock gap makes that lifecycle bug deterministic.
    await new Promise<void>((resolveWait) => setTimeout(resolveWait, 20));
  }
  return {
    decisions: actions.length,
    frames,
    reason,
    success,
    playerStates,
    finalPlayer,
  };
}

function canonicalizeExpertReplay(capture: ExpertReplayCapture): ExpertReplayCapture {
  return {
    ...capture,
    playerStates: capture.playerStates.map((state) =>
      canonicalizePlayerState(state) as Record<string, unknown>),
    finalPlayer: canonicalizePlayerState(capture.finalPlayer),
  };
}

export function createExpertDecisionInputs(
  action: ExpertDecision,
  previousJump: boolean,
): SoakInput[] {
  const moveX = action[0] - 1;
  const moveY = action[1] - 1;
  const jump = action[2] === 1;
  const dash = action[3];
  const grab = action[4] === 1;
  return Array.from({ length: 4 }, (_, frame): SoakInput => ({
    move_x: moveX,
    move_y: moveY,
    jump_pressed: jump && frame === 0 && !previousJump,
    jump_held: jump,
    dash_pressed: dash === 1 && frame === 0,
    crouch_dash_pressed: dash === 2 && frame === 0,
    grab_held: grab,
    talk_pressed: false,
  }));
}

function loadCity2ExpertActions(): ExpertDecision[] {
  const decoded: unknown = JSON.parse(readFileSync(city2ExpertActionsPath, "utf8"));
  if (!isRecord(decoded) || !Array.isArray(decoded.actions)) {
    throw new Error("city2 expert action fixture is malformed");
  }
  return decoded.actions.map((value, index) => {
    if (!Array.isArray(value) || value.length !== 5
        || value.some((component) => !Number.isSafeInteger(component))) {
      throw new Error(`city2 expert action ${index} is malformed`);
    }
    const action = value as number[];
    if (action[0]! < 0 || action[0]! > 2
        || action[1]! < 0 || action[1]! > 2
        || action[2]! < 0 || action[2]! > 1
        || action[3]! < 0 || action[3]! > 2
        || action[4]! < 0 || action[4]! > 1) {
      throw new Error(`city2 expert action ${index} is outside the action space`);
    }
    return action as unknown as ExpertDecision;
  });
}

async function runInputLifecycleGate(
  actors: readonly RunningActor[],
  serviceRoot: string,
  options: ActorOptions,
): Promise<void> {
  const started = performance.now();
  const results = await Promise.all(actors.map(async (actor) => {
    const client = createActorGymClient(actor, serviceRoot);
    for (let round = 0; round < options.inputLifecycleRounds; round++) {
      await assertFirstActionChangesState(client, options, round, "right");
      await assertFirstActionChangesState(client, options, round, "jump");
      // Dash is last on purpose: it leaves hit-stop behind for the next round's
      // reset/right probe, reproducing the long-lived actor failure this gate owns.
      await assertFirstActionChangesState(client, options, round, "dash");
    }
    return {
      actor: actor.index,
      rounds: options.inputLifecycleRounds,
      resets: options.inputLifecycleRounds * 3,
      probes: ["right", "jump", "dash"],
    };
  }));
  process.stdout.write(`${JSON.stringify({
    input_lifecycle_gate: "strict-pass",
    room: options.soakRoom,
    actors: results,
    elapsed_seconds: Number(((performance.now() - started) / 1000).toFixed(2)),
  })}\n`);
}

type InputLifecycleProbe = "right" | "jump" | "dash";

async function assertFirstActionChangesState(
  client: ActorGymClient,
  options: ActorOptions,
  round: number,
  probe: InputLifecycleProbe,
): Promise<void> {
  const reset = await client.gymReset({
    area_id: options.areaId,
    area_mode: options.areaMode,
    ...(options.areaSid ? { area_sid: options.areaSid } : {}),
    room: options.soakRoom,
    seed: round * 3 + (probe === "right" ? 0 : probe === "jump" ? 1 : 2),
    skip_transitions: true,
    max_episode_frames: 32,
    include_entities: false,
    include_player_states: true,
    fast_mode: true,
  });
  const episodeId = reset.observation?.episode_id;
  const before = reset.observation?.player;
  if (typeof episodeId !== "string" || !isRecord(before)) {
    throw new Error(`input lifecycle ${probe} reset returned no player episode`);
  }
  const inputs = createFirstActionProbe(probe);
  const step = await client.gymStep({ episode_id: episodeId, inputs });
  const after = step.observation?.player;
  if (!isRecord(after) || step.frames_executed !== inputs.length) {
    throw new Error(
      `input lifecycle ${probe} round ${round} returned malformed step: ` +
      `frames=${step.frames_executed}`,
    );
  }
  const beforePos = requirePair(before.pos, "reset player.pos");
  const afterPos = requirePair(after.pos, "step player.pos");
  const afterSpeed = requirePair(after.speed, "step player.speed");
  const changed = probe === "right"
    ? afterPos[0] > beforePos[0] || afterSpeed[0] > 0
    : probe === "jump"
      ? afterPos[1] < beforePos[1] || afterSpeed[1] < 0
      : after.state === 2
        || (typeof before.dashes === "number"
          && typeof after.dashes === "number"
          && after.dashes < before.dashes)
        || Math.abs(afterSpeed[0]) >= 100;
  if (!changed) {
    const detail = {
      before: canonicalizePlayerState(before),
      after: canonicalizePlayerState(after),
    };
    throw new Error(
      `input lifecycle ${probe} round ${round} did not affect the real player: ` +
      JSON.stringify(detail),
    );
  }
}

export function createFirstActionProbe(probe: InputLifecycleProbe): SoakInput[] {
  return Array.from({ length: 4 }, (_, frame): SoakInput => ({
    move_x: probe === "right" || probe === "dash" ? 1 : 0,
    move_y: 0,
    jump_pressed: probe === "jump" && frame === 0,
    jump_held: probe === "jump",
    dash_pressed: probe === "dash" && frame === 0,
    crouch_dash_pressed: false,
    grab_held: false,
    talk_pressed: false,
  }));
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === "object" && !Array.isArray(value);
}

function requirePair(value: unknown, path: string): readonly [number, number] {
  if (!Array.isArray(value) || value.length < 2
      || typeof value[0] !== "number" || typeof value[1] !== "number") {
    throw new Error(`${path} is not a numeric pair`);
  }
  return [value[0], value[1]];
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
    area_mode: options.areaMode,
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
          EVEREST_AREA_MODE: String(options.areaMode),
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
          area_mode: options.areaMode,
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
            area_mode: options.areaMode,
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

export function validateDirectGymResponse(response: Record<string, unknown>): ActorGymResult {
  if (response.success !== true) {
    throw new Error(`direct Gym TCP failed: ${String(response.error ?? "unknown error")}`);
  }
  const observation = response.observation;
  if (observation !== undefined
      && (!observation || typeof observation !== "object" || Array.isArray(observation))) {
    throw new Error("direct Gym TCP observation is not an object");
  }
  if (observation !== undefined) {
    const areaMode = (observation as Record<string, unknown>).area_mode;
    if (!Number.isSafeInteger(areaMode) || (areaMode as number) < 0 || (areaMode as number) > 2) {
      throw new Error("direct Gym TCP observation area_mode is invalid");
    }
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
