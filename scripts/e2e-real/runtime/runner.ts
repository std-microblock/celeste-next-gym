import { mkdirSync, readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { spawn, type ChildProcess } from 'node:child_process'

import {
  EVEREST_READY_TIMEOUT_MS,
  SERVICE_PORT_TIMEOUT_MS,
} from '../constants.js'
import type { HarnessConfig } from '../config.js'
import {
  createRunContext,
  reserveLoopbackPort,
  terminateOwnedProcess,
  updateRunManifest,
  validateGameInstall,
  waitForOwnedEverest,
  waitForProcessIdentity,
} from '../isolation/index.js'
import type { GitIdentity, ProcessIdentity, ScenarioDefinition } from '../types.js'
import type { RecordingPlan, RecordingTargetGroup, RecordingArtifactEntry } from '../recording/index.js'
import { captureScenario, encodeScenarioArtifacts, writeArtifactManifest } from '../recording/index.js'
import { createCollectorClient } from './collector-client.js'
import { captureCommand, waitForPort } from './commands.js'
import { createHarnessPaths, prepareMods } from './prepare-mods.js'
import { assertNoPlaygroundMapOverride, groupScenariosByLifecycle, materializePlaygroundMap } from './playground-map.js'
import { compareRealTrace } from './rust-compare.js'
import { executeScenario, type ScenarioSummary } from './scenario-runner.js'
import { writeTrace } from './trace.js'

export interface HarnessSummary {
  readonly health: Readonly<Record<string, unknown>>
  readonly scenarios: readonly ScenarioSummary[]
}

export interface RecordingHarnessSummary {
  readonly recording: true
  readonly scenarioCount: number
  readonly techniqueCount: number
  readonly targets: readonly HarnessSummary[]
}

export async function runRecordingHarness(
  config: HarnessConfig,
  plan: RecordingPlan,
  dependencies: { readonly runTarget?: typeof runHarness } = {},
): Promise<RecordingHarnessSummary> {
  if (!config.ffmpegPath || !config.ffprobePath) throw new Error('recording requires absolute FFMPEG_PATH and FFPROBE_PATH')
  const targets: HarnessSummary[] = []
  const runTarget = dependencies.runTarget ?? runHarness
  for (const group of plan.groups) {
    targets.push(await runTarget(config, group.scenarios.map((item) => item.scenario), group))
  }
  return { recording: true, scenarioCount: plan.scenarioCount, techniqueCount: plan.techniqueCount, targets }
}

export async function runHarness(
  config: HarnessConfig,
  scenarios: readonly ScenarioDefinition[],
  recordingGroup?: RecordingTargetGroup,
): Promise<HarnessSummary> {
  if (scenarios.length === 0) throw new Error('no E2E scenarios selected')
  const targets = new Set(scenarios.map((scenario) => scenario.target.id))
  if (targets.size !== 1) throw new Error(`one E2E invocation cannot mix targets: ${[...targets].join(', ')}`)
  const lifecycleGroups = groupScenariosByLifecycle(scenarios)
  if (lifecycleGroups.length === 1) {
    return await runHarnessLifecycle(config, scenarios, recordingGroup)
  }
  const results: HarnessSummary[] = []
  for (const lifecycle of lifecycleGroups) {
    const names = new Set(lifecycle.scenarios.map((scenario) => scenario.name))
    const lifecycleRecordingGroup = recordingGroup ? Object.freeze({
      target: recordingGroup.target,
      scenarios: Object.freeze(recordingGroup.scenarios.filter((item) => names.has(item.scenario.name))),
    }) : undefined
    results.push(await runHarnessLifecycle(config, lifecycle.scenarios, lifecycleRecordingGroup))
  }
  return {
    health: Object.freeze({ lifecycles: results.map((result) => result.health) }),
    scenarios: Object.freeze(results.flatMap((result) => result.scenarios)),
  }
}

async function runHarnessLifecycle(
  config: HarnessConfig,
  scenarios: readonly ScenarioDefinition[],
  recordingGroup?: RecordingTargetGroup,
): Promise<HarnessSummary> {
  if (scenarios.length === 0) throw new Error('no E2E scenarios selected')
  const targets = new Set(scenarios.map((scenario) => scenario.target.id))
  if (targets.size !== 1) throw new Error(`one E2E invocation cannot mix targets: ${[...targets].join(', ')}`)
  const scenarioTarget = scenarios[0]?.target
  if (!scenarioTarget) throw new Error('scenario target is missing')
  assertNoPlaygroundMapOverride(scenarios[0]!, config.mapPath)

  const paths = createHarnessPaths(config.repoRoot)
  const gameInstall = validateGameInstall({
    repoRoot: config.repoRoot,
    gameRoot: paths.gameRoot,
    steamRoots: config.steamRoots,
  })
  const git: GitIdentity = {
    branch: captureCommand('git', ['branch', '--show-current'], config.repoRoot),
    head: captureCommand('git', ['rev-parse', 'HEAD'], config.repoRoot),
  }
  validateExpectedGit(config, git)

  const modPortReservation = await reserveLoopbackPort()
  const httpPortReservation = await reserveLoopbackPort()
  const runContext = createRunContext({
    repoRoot: config.repoRoot,
    gameInstall,
    modPort: modPortReservation.port,
    httpPort: httpPortReservation.port,
    git,
  })
  const recordingRoot = resolve(runContext.runRoot, 'recordings')
  if (recordingGroup) mkdirSync(recordingRoot, { recursive: true })
  let game: ChildProcess | undefined
  let service: ChildProcess | undefined
  let gameIdentity: ProcessIdentity | undefined
  let serviceIdentity: ProcessIdentity | undefined
  let runError: unknown
  try {
    const materializedMap = scenarioTarget.kind === 'playground'
      ? materializePlaygroundMap({
        repoRoot: config.repoRoot,
        runRoot: runContext.runRoot,
        paths,
        scenario: scenarios[0]!,
      })
      : undefined
    prepareMods(paths, gameInstall, materializedMap?.modRoot)
    await modPortReservation.release()
    updateRunManifest(runContext, { status: 'starting-game' })
    game = spawn(gameInstall.executable, ['--disable-splash', '--loglevel', 'info'], {
      cwd: gameInstall.gameRoot,
      windowsHide: !config.showGameWindow,
      stdio: 'ignore',
      shell: false,
      env: {
        ...process.env,
        CELESTE_GYM_COLLECTOR_PORT: String(modPortReservation.port),
        CELESTE_GYM_RUN_NONCE: runContext.runNonce,
        EVEREST_SAVEPATH: runContext.saveRoot,
        EVEREST_TMPDIR: runContext.tempRoot,
        ...(recordingGroup ? { CELESTE_GYM_RECORDING_ROOT: recordingRoot } : {}),
      },
    })
    if (!game.pid) throw new Error('Celeste child did not expose a process id')
    gameIdentity = await waitForProcessIdentity(game.pid, gameInstall.executable)
    updateRunManifest(runContext, { status: 'waiting-for-everest', game_process: gameIdentity })
    const everestPing = await waitForOwnedEverest(modPortReservation.port, {
      runNonce: runContext.runNonce,
      processId: game.pid,
      port: modPortReservation.port,
    }, EVEREST_READY_TIMEOUT_MS)
    updateRunManifest(runContext, { status: 'game-authenticated', everest_ping: everestPing })

    await httpPortReservation.release()
    service = spawn(process.execPath, [resolve(paths.serviceRoot, 'dist', 'src', 'index.js')], {
      cwd: paths.serviceRoot,
      windowsHide: true,
      stdio: 'ignore',
      shell: false,
      env: {
        ...process.env,
        COLLECTOR_BACKEND: 'everest',
        COLLECTOR_PORT: String(httpPortReservation.port),
        COLLECTOR_TIMEOUT_MS: '60000',
        EVEREST_COLLECTOR_PORT: String(modPortReservation.port),
        ...collectorOwnershipEnvironment(runContext.runNonce, gameIdentity.processId),
        EVEREST_AREA_ID: String(scenarioTarget.areaId),
        ...(scenarioTarget.kind === 'playground'
          ? { EVEREST_AREA_SID: scenarioTarget.areaSid }
          : config.areaSid ? { EVEREST_AREA_SID: config.areaSid } : {}),
      },
    })
    if (!service.pid) throw new Error('collector service child did not expose a process id')
    serviceIdentity = await waitForProcessIdentity(service.pid, process.execPath)
    updateRunManifest(runContext, { status: 'waiting-for-service', service_process: serviceIdentity })
    await waitForPort(httpPortReservation.port, SERVICE_PORT_TIMEOUT_MS)

    const client = createCollectorClient(paths.serviceRoot, httpPortReservation.port)
    const health = await client.waitUntilReady()
    const mapPath = materializedMap?.mapPath ?? resolveMapPath(config, paths, scenarioTarget)
    const map = readFileSync(mapPath)
    const summaries: ScenarioSummary[] = []
    const recordingArtifacts: RecordingArtifactEntry[] = []
    for (const scenario of scenarios) {
      const recordingItem = recordingGroup?.scenarios.find((item) => item.scenario.name === scenario.name)
      if (!recordingItem) {
        summaries.push(await runOneScenario(scenario))
        continue
      }
      if (!game.pid || !gameIdentity) throw new Error('recording requires an authenticated game process')
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
          const sessionDirectory = resolve(recordingRoot, 'scenarios', scenario.name, captureToken)
          mkdirSync(sessionDirectory, { recursive: true })
          return await runOneScenario(scenario, captureToken, resolve(sessionDirectory, 'trace.json'))
        },
      })
      summaries.push(captured.execution)
      const media = requireMediaPaths(config)
      recordingArtifacts.push(...await encodeScenarioArtifacts({
        item: recordingItem, manifestPath: captured.presentation.manifestPath,
        tracePath: captured.execution.tracePath, recordingRoot, ...media,
      }))
    }
    if (recordingGroup && gameIdentity) {
      const artifactsManifest = await writeArtifactManifest({
        recordingRoot, runNonce: runContext.runNonce, gameProcess: gameIdentity, artifacts: recordingArtifacts,
      })
      updateRunManifest(runContext, { recording: { root: recordingRoot, artifacts_manifest: artifactsManifest, artifacts: recordingArtifacts } })
    }
    updateRunManifest(runContext, { status: 'completed' })
    return { health, scenarios: summaries }

    async function runOneScenario(scenario: ScenarioDefinition, captureToken?: string, tracePath?: string): Promise<ScenarioSummary> {
      return await executeScenario({
        scenario, map, mapPath, repoRoot: config.repoRoot,
        ...(config.room ? { room: config.room } : {}),
        skipTransitions: config.skipTransitions, collectOnly: config.collectOnly,
        ...(captureToken ? { captureToken } : {}), ...(tracePath ? { tracePath } : {}),
        dependencies: {
          simulate: async (request) => await client.simulate(request), writeTrace,
          compare: (options) => compareRealTrace({ repoRoot: config.repoRoot, ...options }),
        },
      })
    }

  } catch (error) {
    runError = error
    updateRunManifest(runContext, { status: 'failed', error: String(error) })
    throw error
  } finally {
    await Promise.allSettled([modPortReservation.release(), httpPortReservation.release()])
    const cleanup = {
      service_terminated: cleanupOwned('collector', service, serviceIdentity),
      game_terminated: cleanupOwned('Celeste', game, gameIdentity),
    }
    updateRunManifest(runContext, {
      status: runError ? 'failed-cleanup-finished' : 'cleanup-finished',
      cleanup,
    })
  }
}

export function collectorOwnershipEnvironment(runNonce: string, processId: number): Readonly<Record<string, string>> {
  if (!runNonce) throw new Error('collector ownership requires a run nonce')
  if (!Number.isSafeInteger(processId) || processId <= 0) throw new Error('collector ownership requires a positive process id')
  return Object.freeze({ EVEREST_RUN_NONCE: runNonce, EVEREST_PROCESS_ID: String(processId) })
}

function requireMediaPaths(config: HarnessConfig): { ffmpegPath: string; ffprobePath: string } {
  if (!config.ffmpegPath || !config.ffprobePath) throw new Error('recording requires FFMPEG_PATH and FFPROBE_PATH')
  return { ffmpegPath: config.ffmpegPath, ffprobePath: config.ffprobePath }
}


function resolveMapPath(
  config: HarnessConfig,
  paths: ReturnType<typeof createHarnessPaths>,
  target: ScenarioDefinition['target'],
): string {
  if (config.mapPath) return config.mapPath
  return target.kind === 'playground'
    ? resolve(paths.playgroundModRoot, 'Maps', 'CelesteGymPlayground', 'Playground.bin')
    : resolve(paths.gameRoot, 'Content', 'Maps', target.defaultMapFile)
}

function validateExpectedGit(config: HarnessConfig, git: GitIdentity): void {
  if (config.expectedGitBranch && git.branch !== config.expectedGitBranch) {
    throw new Error(`expected git branch ${config.expectedGitBranch}, got ${git.branch || '(detached)'}`)
  }
  if (config.expectedGitHead && git.head !== config.expectedGitHead) {
    throw new Error(`expected git HEAD ${config.expectedGitHead}, got ${git.head}`)
  }
}

function cleanupOwned(
  label: string,
  child: ChildProcess | undefined,
  identity: ProcessIdentity | undefined,
): boolean {
  if (!identity) {
    if (child?.pid && child.exitCode === null) {
      console.warn(`refusing to terminate ${label} PID ${child.pid}: no recorded process identity`)
    }
    return false
  }
  try {
    return terminateOwnedProcess({ child, expectedIdentity: identity })
  } catch (error) {
    console.warn(`failed to terminate owned ${label} PID ${child?.pid}: ${String(error)}`)
    return false
  }
}
