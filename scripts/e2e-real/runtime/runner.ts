import { existsSync, mkdirSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { spawn, type ChildProcess } from "node:child_process";

import {
  EVEREST_READY_TIMEOUT_MS,
  SERVICE_PORT_TIMEOUT_MS,
} from "../constants.js";
import type { HarnessConfig } from "../config.js";
import {
  createRunContext,
  reserveLoopbackPort,
  terminateOwnedProcess,
  updateRunManifest,
  validateGameInstall,
  waitForOwnedEverest,
  waitForProcessIdentity,
} from "../isolation/index.js";
import type {
  GitIdentity,
  ProcessIdentity,
  ScenarioDefinition,
} from "../types.js";
import type {
  RecordingPlan,
  RecordingTargetGroup,
  RecordingArtifactEntry,
} from "../recording/index.js";
import {
  captureScenario,
  encodeScenarioArtifacts,
  writeArtifactManifest,
} from "../recording/index.js";
import { createCollectorClient } from "./collector-client.js";
import { captureCommand, waitForPort } from "./commands.js";
import { createHarnessPaths, prepareMods } from "./prepare-mods.js";
import {
  assertNoPlaygroundMapOverride,
  groupScenariosByLifecycle,
  materializePlaygroundMap,
} from "./playground-map.js";
import { compareRealTrace } from "./rust-compare.js";
import { executeScenario, type ScenarioSummary } from "./scenario-runner.js";
import { writeTrace } from "./trace.js";

export interface HarnessSummary {
  readonly health: Readonly<Record<string, unknown>>;
  readonly scenarios: readonly ScenarioSummary[];
}

export interface RecordingHarnessSummary {
  readonly recording: true;
  readonly scenarioCount: number;
  readonly techniqueCount: number;
  readonly targets: readonly HarnessSummary[];
}

export async function runRecordingHarness(
  config: HarnessConfig,
  plan: RecordingPlan,
  dependencies: { readonly runTarget?: typeof runHarness } = {},
): Promise<RecordingHarnessSummary> {
  if (!config.ffmpegPath || !config.ffprobePath)
    throw new Error("recording requires absolute FFMPEG_PATH and FFPROBE_PATH");
  const targets: HarnessSummary[] = [];
  const runTarget = dependencies.runTarget ?? runHarness;
  for (const group of plan.groups) {
    targets.push(
      await runTarget(
        config,
        group.scenarios.map((item) => item.scenario),
        group,
      ),
    );
  }
  return {
    recording: true,
    scenarioCount: plan.scenarioCount,
    techniqueCount: plan.techniqueCount,
    targets,
  };
}

export async function runHarness(
  config: HarnessConfig,
  scenarios: readonly ScenarioDefinition[],
  recordingGroup?: RecordingTargetGroup,
): Promise<HarnessSummary> {
  if (scenarios.length === 0) throw new Error("no E2E scenarios selected");
  const targets = new Set(scenarios.map((scenario) => scenario.target.id));
  if (targets.size !== 1)
    throw new Error(
      `one E2E invocation cannot mix targets: ${[...targets].join(", ")}`,
    );
  const lifecycleGroups = groupScenariosByLifecycle(scenarios);
  if (lifecycleGroups.length === 1) {
    return await runHarnessLifecycle(config, scenarios, recordingGroup);
  }
  const results: HarnessSummary[] = [];
  for (const lifecycle of lifecycleGroups) {
    const names = new Set(lifecycle.scenarios.map((scenario) => scenario.name));
    const lifecycleRecordingGroup = recordingGroup
      ? Object.freeze({
          target: recordingGroup.target,
          scenarios: Object.freeze(
            recordingGroup.scenarios.filter((item) =>
              names.has(item.scenario.name),
            ),
          ),
        })
      : undefined;
    results.push(
      await runHarnessLifecycle(
        config,
        lifecycle.scenarios,
        lifecycleRecordingGroup,
      ),
    );
  }
  return {
    health: Object.freeze({
      lifecycles: results.map((result) => result.health),
    }),
    scenarios: Object.freeze(results.flatMap((result) => result.scenarios)),
  };
}

async function runHarnessLifecycle(
  config: HarnessConfig,
  scenarios: readonly ScenarioDefinition[],
  recordingGroup?: RecordingTargetGroup,
): Promise<HarnessSummary> {
  if (scenarios.length === 0) throw new Error("no E2E scenarios selected");
  const targets = new Set(scenarios.map((scenario) => scenario.target.id));
  if (targets.size !== 1)
    throw new Error(
      `one E2E invocation cannot mix targets: ${[...targets].join(", ")}`,
    );
  const scenarioTarget = scenarios[0]?.target;
  if (!scenarioTarget) throw new Error("scenario target is missing");
  assertNoPlaygroundMapOverride(scenarios[0]!, config.mapPath);

  const paths = createHarnessPaths(config.repoRoot);
  const gameInstall = validateGameInstall({
    repoRoot: config.repoRoot,
    gameRoot: paths.gameRoot,
    steamRoots: config.steamRoots,
  });
  const git: GitIdentity = {
    branch: captureCommand(
      "git",
      ["branch", "--show-current"],
      config.repoRoot,
    ),
    head: captureCommand("git", ["rev-parse", "HEAD"], config.repoRoot),
  };
  validateExpectedGit(config, git);

  const modPortReservation = await reserveLoopbackPort();
  const httpPortReservation = await reserveLoopbackPort();
  const runContext = createRunContext({
    repoRoot: config.repoRoot,
    gameInstall,
    modPort: modPortReservation.port,
    httpPort: httpPortReservation.port,
    git,
  });
  const recordingRoot = resolve(runContext.runRoot, "recordings");
  if (recordingGroup) mkdirSync(recordingRoot, { recursive: true });
  let game: ChildProcess | undefined;
  let service: ChildProcess | undefined;
  let gameIdentity: ProcessIdentity | undefined;
  let serviceIdentity: ProcessIdentity | undefined;
  let runError: unknown;
  try {
    const materializedMap =
      scenarioTarget.kind === "playground"
        ? materializePlaygroundMap({
            repoRoot: config.repoRoot,
            runRoot: runContext.runRoot,
            paths,
            scenario: scenarios[0]!,
          })
        : undefined;
    prepareMods(paths, gameInstall, materializedMap?.modRoot);
    await modPortReservation.release();
    updateRunManifest(runContext, { status: "starting-game" });
    game = spawn(
      gameInstall.executable,
      ["--disable-splash", "--loglevel", "info"],
      {
        cwd: gameInstall.gameRoot,
        windowsHide: !config.showGameWindow,
        stdio: "ignore",
        shell: false,
        env: {
          ...process.env,
          CELESTE_GYM_COLLECTOR_PORT: String(modPortReservation.port),
          CELESTE_GYM_RUN_NONCE: runContext.runNonce,
          EVEREST_SAVEPATH: runContext.saveRoot,
          EVEREST_TMPDIR: runContext.tempRoot,
          ...(recordingGroup
            ? { CELESTE_GYM_RECORDING_ROOT: recordingRoot }
            : {}),
        },
      },
    );
    if (!game.pid) throw new Error("Celeste child did not expose a process id");
    gameIdentity = await waitForProcessIdentity(
      game.pid,
      gameInstall.executable,
    );
    updateRunManifest(runContext, {
      status: "waiting-for-everest",
      game_process: gameIdentity,
    });
    const everestPing = await waitForOwnedEverest(
      modPortReservation.port,
      {
        runNonce: runContext.runNonce,
        processId: game.pid,
        port: modPortReservation.port,
      },
      EVEREST_READY_TIMEOUT_MS,
    );
    updateRunManifest(runContext, {
      status: "game-authenticated",
      everest_ping: everestPing,
    });

    await httpPortReservation.release();
    service = spawn(
      process.execPath,
      [resolve(paths.serviceRoot, "dist", "src", "index.js")],
      {
        cwd: paths.serviceRoot,
        windowsHide: true,
        stdio: "ignore",
        shell: false,
        env: {
          ...process.env,
          COLLECTOR_BACKEND: "everest",
          COLLECTOR_PORT: String(httpPortReservation.port),
          COLLECTOR_TIMEOUT_MS: "60000",
          EVEREST_COLLECTOR_PORT: String(modPortReservation.port),
          ...collectorOwnershipEnvironment(
            runContext.runNonce,
            gameIdentity.processId,
          ),
          EVEREST_AREA_ID: String(scenarioTarget.areaId),
          ...(scenarioTarget.kind === "playground"
            ? { EVEREST_AREA_SID: scenarioTarget.areaSid }
            : config.areaSid
              ? { EVEREST_AREA_SID: config.areaSid }
              : {}),
        },
      },
    );
    if (!service.pid)
      throw new Error("collector service child did not expose a process id");
    serviceIdentity = await waitForProcessIdentity(
      service.pid,
      process.execPath,
    );
    updateRunManifest(runContext, {
      status: "waiting-for-service",
      service_process: serviceIdentity,
    });
    await waitForPort(httpPortReservation.port, SERVICE_PORT_TIMEOUT_MS);

    const client = createCollectorClient(
      paths.serviceRoot,
      httpPortReservation.port,
    );
    const baseHealth = await client.waitUntilReady();
    const gymSmokeRoom = config.room ?? scenarios[0]?.room;
    const fastGymSmoke = process.env.E2E_GYM_FAST_SMOKE === "1";
    const gymSmoke =
      process.env.E2E_GYM_SMOKE === "1" || fastGymSmoke
        ? await runGymSmoke(client, {
            areaId: scenarioTarget.areaId,
            ...(scenarioTarget.kind === "playground"
              ? { areaSid: scenarioTarget.areaSid }
              : config.areaSid
                ? { areaSid: config.areaSid }
                : {}),
            ...(gymSmokeRoom ? { room: gymSmokeRoom } : {}),
          }, fastGymSmoke)
        : undefined;
    const health = Object.freeze({
      ...baseHealth,
      ...(gymSmoke === undefined ? {} : { gym_smoke: gymSmoke }),
    });
    const mapPath =
      materializedMap?.mapPath ?? resolveMapPath(config, paths, scenarioTarget);
    const map = readFileSync(mapPath);
    const summaries: ScenarioSummary[] = [];
    const recordingArtifacts: RecordingArtifactEntry[] = [];
    for (const scenario of scenarios) {
      const recordingItem = recordingGroup?.scenarios.find(
        (item) => item.scenario.name === scenario.name,
      );
      if (!recordingItem) {
        summaries.push(await runOneScenario(scenario));
        continue;
      }
      if (!game.pid || !gameIdentity)
        throw new Error("recording requires an authenticated game process");
      const captured = await captureScenario({
        service: client,
        recordingRoot,
        scenarioId: scenario.name,
        endStateIndex: scenario.inputs.length,
        runNonce: runContext.runNonce,
        gameProcessId: game.pid,
        timeoutMs: config.recordingTimeoutMs,
        pollTimeoutMs: config.recordingPollTimeoutMs,
        execute: async (captureToken) => {
          const sessionDirectory = resolve(
            recordingRoot,
            "scenarios",
            scenario.name,
            captureToken,
          );
          mkdirSync(sessionDirectory, { recursive: true });
          return await runOneScenario(
            scenario,
            captureToken,
            resolve(sessionDirectory, "trace.json"),
          );
        },
      });
      summaries.push(captured.execution);
      const media = requireMediaPaths(config);
      recordingArtifacts.push(
        ...(await encodeScenarioArtifacts({
          item: recordingItem,
          manifestPath: captured.presentation.manifestPath,
          tracePath: captured.execution.tracePath,
          recordingRoot,
          ...media,
        })),
      );
    }
    if (recordingGroup && gameIdentity) {
      const artifactsManifest = await writeArtifactManifest({
        recordingRoot,
        runNonce: runContext.runNonce,
        gameProcess: gameIdentity,
        artifacts: recordingArtifacts,
      });
      updateRunManifest(runContext, {
        recording: {
          root: recordingRoot,
          artifacts_manifest: artifactsManifest,
          artifacts: recordingArtifacts,
        },
      });
    }
    updateRunManifest(runContext, { status: "completed" });
    return { health, scenarios: summaries };

    async function runOneScenario(
      scenario: ScenarioDefinition,
      captureToken?: string,
      tracePath?: string,
    ): Promise<ScenarioSummary> {
      // Keep non-recording traces in the immutable per-run root as well: the
      // old shared `.tmp/e2e-<scenario>-trace.json` path is overwritten by the
      // next run and cannot prove which generated fixture produced a trace.
      const resolvedTracePath =
        tracePath ??
        resolve(runContext.runRoot, "traces", `${scenario.name}.json`);
      const traceEntry = {
        scenario: scenario.name,
        path: resolvedTracePath,
        map_path: mapPath,
        ...((config.room ?? scenario.room)
          ? { room: config.room ?? scenario.room }
          : {}),
      };
      try {
        return await executeScenario({
          scenario,
          map,
          mapPath,
          repoRoot: config.repoRoot,
          ...(config.room ? { room: config.room } : {}),
          skipTransitions: config.skipTransitions,
          collectOnly: config.collectOnly,
          ...(captureToken ? { captureToken } : {}),
          tracePath: resolvedTracePath,
          dependencies: {
            simulate: async (request) => await client.simulate(request),
            writeTrace,
            compare: (options) =>
              compareRealTrace({ repoRoot: config.repoRoot, ...options }),
          },
        });
      } finally {
        // executeScenario writes the trace before semantic verification and
        // Rust comparison.  Persist the binding even when either later guard
        // fails, provided the trace was actually written.
        if (existsSync(resolvedTracePath)) {
          const traces = Array.isArray(runContext.manifest.traces)
            ? runContext.manifest.traces
            : [];
          updateRunManifest(runContext, { traces: [...traces, traceEntry] });
        }
      }
    }
  } catch (error) {
    runError = error;
    updateRunManifest(runContext, { status: "failed", error: String(error) });
    throw error;
  } finally {
    await Promise.allSettled([
      modPortReservation.release(),
      httpPortReservation.release(),
    ]);
    const [serviceTerminated, gameTerminated] = await Promise.all([
      cleanupOwned("collector", service, serviceIdentity),
      cleanupOwned("Celeste", game, gameIdentity),
    ]);
    const cleanup = {
      service_terminated: serviceTerminated,
      game_terminated: gameTerminated,
    };
    updateRunManifest(runContext, {
      status: runError ? "failed-cleanup-finished" : "cleanup-finished",
      cleanup,
    });
  }
}

async function runGymSmoke(
  client: ReturnType<typeof createCollectorClient>,
  target: { readonly areaId: number; readonly areaSid?: string; readonly room?: string },
  fastMode: boolean,
): Promise<Readonly<Record<string, unknown>>> {
  if (fastMode) return await runFastGymSmoke(client, target);
  const resetStarted = performance.now();
  const reset = await client.gymReset({
    area_id: target.areaId,
    ...(target.areaSid ? { area_sid: target.areaSid } : {}),
    ...(target.room ? { room: target.room } : {}),
    skip_transitions: true,
    max_episode_frames: 600,
    include_entities: true,
    include_player_states: true,
    fast_mode: false,
  });
  const resetMs = performance.now() - resetStarted;
  const initial = requireGymObservation(reset.observation, "gym reset");
  const episodeId = initial.episode_id;
  if (typeof episodeId !== "string")
    throw new Error("gym reset observation has no episode_id");
  const geometry = initial.room_geometry;
  if (!geometry || typeof geometry !== "object" || Array.isArray(geometry))
    throw new Error("gym reset observation has no room_geometry");
  if ((geometry as Record<string, unknown>).tile_size !== 8)
    throw new Error("gym room geometry is not 8px aligned");
  if (!Array.isArray(initial.entities))
    throw new Error("gym reset observation has no entity array");

  const inputs = Array.from({ length: 16 }, () => ({
    move_x: 0,
    move_y: 0,
    jump_pressed: false,
    jump_held: false,
    dash_pressed: false,
    crouch_dash_pressed: false,
    grab_held: false,
    talk_pressed: false,
  }));
  const stepStarted = performance.now();
  const stepped = await client.gymStep({ episode_id: episodeId, inputs });
  const stepMs = performance.now() - stepStarted;
  const final = requireGymObservation(stepped.observation, "gym step");
  if (stepped.frames_executed <= 0)
    throw new Error("gym step executed no frames");
  await client.gymObserve({ episode_id: episodeId });
  await client.gymClose({ episode_id: episodeId });
  return Object.freeze({
    episode_id: episodeId,
    room: initial.room,
    tile_width: (geometry as Record<string, unknown>).width,
    tile_height: (geometry as Record<string, unknown>).height,
    entity_count: initial.entities.length,
    frames_executed: stepped.frames_executed,
    episode_frame: final.episode_frame,
    reset_ms: Number(resetMs.toFixed(1)),
    step_ms: Number(stepMs.toFixed(1)),
    effective_fps: Number(
      ((stepped.frames_executed * 1000) / Math.max(stepMs, 0.001)).toFixed(1),
    ),
  });
}

async function runFastGymSmoke(
  client: ReturnType<typeof createCollectorClient>,
  target: { readonly areaId: number; readonly areaSid?: string; readonly room?: string },
): Promise<Readonly<Record<string, unknown>>> {
  const deterministicInputs = neutralGymInputs(120);
  const baseline = await runMeasuredGymBatch(client, target, {
    fastMode: false,
    inputs: deterministicInputs,
    includeEntities: false,
  });
  const accelerated = await runMeasuredGymBatch(client, target, {
    fastMode: true,
    inputs: deterministicInputs,
    includeEntities: false,
  });
  if (baseline.framesExecuted !== deterministicInputs.length) {
    throw new Error(
      `normal gym determinism run stopped after ${baseline.framesExecuted} frames`,
    );
  }
  if (accelerated.framesExecuted !== deterministicInputs.length) {
    throw new Error(
      `fast gym determinism run stopped after ${accelerated.framesExecuted} frames`,
    );
  }
  compareGymPlayers(baseline.final.player, accelerated.final.player);

  const throughputInputs = neutralGymInputs(1024);
  const throughput = await runMeasuredGymBatch(client, target, {
    fastMode: true,
    inputs: throughputInputs,
    includeEntities: false,
  });
  if (throughput.framesExecuted !== throughputInputs.length) {
    throw new Error(
      `fast gym throughput run stopped after ${throughput.framesExecuted} frames`,
    );
  }
  if (throughput.effectiveFps <= 60) {
    throw new Error(
      `fast gym loop did not exceed 60 physics FPS: ${throughput.effectiveFps}`,
    );
  }

  return Object.freeze({
    mode: "fast",
    deterministic_frames: deterministicInputs.length,
    deterministic_match: true,
    normal_step_ms: Number(baseline.stepMs.toFixed(1)),
    normal_effective_fps: Number(baseline.effectiveFps.toFixed(1)),
    fast_step_ms: Number(accelerated.stepMs.toFixed(1)),
    fast_effective_fps: Number(accelerated.effectiveFps.toFixed(1)),
    throughput_frames: throughput.framesExecuted,
    throughput_step_ms: Number(throughput.stepMs.toFixed(1)),
    throughput_effective_fps: Number(throughput.effectiveFps.toFixed(1)),
    speedup_over_normal: Number(
      (throughput.effectiveFps / Math.max(baseline.effectiveFps, 0.001)).toFixed(2),
    ),
  });
}

async function runMeasuredGymBatch(
  client: ReturnType<typeof createCollectorClient>,
  target: { readonly areaId: number; readonly areaSid?: string; readonly room?: string },
  options: {
    readonly fastMode: boolean;
    readonly inputs: readonly Record<string, unknown>[];
    readonly includeEntities: boolean;
  },
): Promise<{
  readonly final: Record<string, unknown>;
  readonly framesExecuted: number;
  readonly stepMs: number;
  readonly effectiveFps: number;
}> {
  const reset = await client.gymReset({
    area_id: target.areaId,
    ...(target.areaSid ? { area_sid: target.areaSid } : {}),
    ...(target.room ? { room: target.room } : {}),
    skip_transitions: true,
    max_episode_frames: Math.max(options.inputs.length + 60, 600),
    include_entities: options.includeEntities,
    include_player_states: false,
    fast_mode: options.fastMode,
  });
  const initial = requireGymObservation(reset.observation, "gym reset");
  if (initial.fast_mode !== options.fastMode) {
    throw new Error("gym reset did not preserve the requested fast_mode");
  }
  const episodeId = initial.episode_id;
  if (typeof episodeId !== "string") {
    throw new Error("gym reset observation has no episode_id");
  }

  const started = performance.now();
  const stepped = await client.gymStep({
    episode_id: episodeId,
    inputs: [...options.inputs],
  });
  const stepMs = performance.now() - started;
  const final = requireGymObservation(stepped.observation, "gym step");
  await client.gymClose({ episode_id: episodeId });
  return {
    final,
    framesExecuted: stepped.frames_executed,
    stepMs,
    effectiveFps: (stepped.frames_executed * 1000) / Math.max(stepMs, 0.001),
  };
}

function neutralGymInputs(count: number): readonly Record<string, unknown>[] {
  return Array.from({ length: count }, () => ({
    move_x: 0,
    move_y: 0,
    jump_pressed: false,
    jump_held: false,
    dash_pressed: false,
    crouch_dash_pressed: false,
    grab_held: false,
    talk_pressed: false,
  }));
}

function compareGymPlayers(
  expectedValue: unknown,
  actualValue: unknown,
): void {
  if (
    typeof expectedValue !== "object" ||
    expectedValue === null ||
    Array.isArray(expectedValue) ||
    typeof actualValue !== "object" ||
    actualValue === null ||
    Array.isArray(actualValue)
  ) {
    throw new Error("gym determinism comparison requires player objects");
  }
  const expected = expectedValue as Record<string, unknown>;
  const actual = actualValue as Record<string, unknown>;
  for (const field of ["pos", "speed"] as const) {
    const left = expected[field];
    const right = actual[field];
    if (!Array.isArray(left) || !Array.isArray(right) || left.length < 2 || right.length < 2) {
      throw new Error(`gym determinism comparison has invalid ${field}`);
    }
    for (let axis = 0; axis < 2; axis++) {
      if (
        typeof left[axis] !== "number" ||
        typeof right[axis] !== "number" ||
        Math.abs(left[axis] - right[axis]) > 0.01
      ) {
        throw new Error(
          `fast gym ${field}[${axis}] differs: normal=${left[axis]} fast=${right[axis]}`,
        );
      }
    }
  }
  for (const field of [
    "state",
    "facing",
    "dashes",
    "stamina",
    "on_ground",
    "ducking",
    "dead",
  ] as const) {
    const difference =
      typeof expected[field] === "number" && typeof actual[field] === "number"
        ? Math.abs(expected[field] - actual[field])
        : expected[field] === actual[field]
          ? 0
          : Number.POSITIVE_INFINITY;
    if (difference > 0.01) {
      throw new Error(
        `fast gym ${field} differs: normal=${String(expected[field])} fast=${String(actual[field])}`,
      );
    }
  }
}

function requireGymObservation(
  value: Record<string, unknown> | undefined,
  label: string,
): Record<string, unknown> {
  if (value === undefined) throw new Error(`${label} returned no observation`);
  return value;
}

export function collectorOwnershipEnvironment(
  runNonce: string,
  processId: number,
): Readonly<Record<string, string>> {
  if (!runNonce) throw new Error("collector ownership requires a run nonce");
  if (!Number.isSafeInteger(processId) || processId <= 0)
    throw new Error("collector ownership requires a positive process id");
  return Object.freeze({
    EVEREST_RUN_NONCE: runNonce,
    EVEREST_PROCESS_ID: String(processId),
  });
}

function requireMediaPaths(config: HarnessConfig): {
  ffmpegPath: string;
  ffprobePath: string;
} {
  if (!config.ffmpegPath || !config.ffprobePath)
    throw new Error("recording requires FFMPEG_PATH and FFPROBE_PATH");
  return { ffmpegPath: config.ffmpegPath, ffprobePath: config.ffprobePath };
}

function resolveMapPath(
  config: HarnessConfig,
  paths: ReturnType<typeof createHarnessPaths>,
  target: ScenarioDefinition["target"],
): string {
  if (config.mapPath) return config.mapPath;
  return target.kind === "playground"
    ? resolve(
        paths.playgroundModRoot,
        "Maps",
        "CelesteGymPlayground",
        "Playground.bin",
      )
    : resolve(paths.gameRoot, "Content", "Maps", target.defaultMapFile);
}

function validateExpectedGit(config: HarnessConfig, git: GitIdentity): void {
  if (config.expectedGitBranch && git.branch !== config.expectedGitBranch) {
    throw new Error(
      `expected git branch ${config.expectedGitBranch}, got ${git.branch || "(detached)"}`,
    );
  }
  if (config.expectedGitHead && git.head !== config.expectedGitHead) {
    throw new Error(
      `expected git HEAD ${config.expectedGitHead}, got ${git.head}`,
    );
  }
}

async function cleanupOwned(
  label: string,
  child: ChildProcess | undefined,
  identity: ProcessIdentity | undefined,
): Promise<boolean> {
  if (!identity) {
    if (child?.pid && child.exitCode === null) {
      console.warn(
        `refusing to terminate ${label} PID ${child.pid}: no recorded process identity`,
      );
    }
    return false;
  }
  try {
    return await terminateOwnedProcess({ child, expectedIdentity: identity });
  } catch (error) {
    console.warn(
      `failed to terminate owned ${label} PID ${child?.pid}: ${String(error)}`,
    );
    return false;
  }
}
