import { readFileSync } from 'node:fs'
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
import { createCollectorClient } from './collector-client.js'
import { captureCommand, waitForPort } from './commands.js'
import { createHarnessPaths, prepareMods } from './prepare-mods.js'
import { compareRealTrace } from './rust-compare.js'
import { executeScenario, type ScenarioSummary } from './scenario-runner.js'
import { writeTrace } from './trace.js'

export interface HarnessSummary {
  readonly health: Readonly<Record<string, unknown>>
  readonly scenarios: readonly ScenarioSummary[]
}

export async function runHarness(
  config: HarnessConfig,
  scenarios: readonly ScenarioDefinition[],
): Promise<HarnessSummary> {
  if (scenarios.length === 0) throw new Error('no E2E scenarios selected')
  const targets = new Set(scenarios.map((scenario) => scenario.target.id))
  if (targets.size !== 1) throw new Error(`one E2E invocation cannot mix targets: ${[...targets].join(', ')}`)
  const scenarioTarget = scenarios[0]?.target
  if (!scenarioTarget) throw new Error('scenario target is missing')

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
  let game: ChildProcess | undefined
  let service: ChildProcess | undefined
  let gameIdentity: ProcessIdentity | undefined
  let serviceIdentity: ProcessIdentity | undefined
  let runError: unknown
  try {
    prepareMods(paths, gameInstall)
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
        EVEREST_AREA_ID: String(scenarioTarget.areaId),
        ...(config.areaSid ? { EVEREST_AREA_SID: config.areaSid } : {}),
      },
    })
    if (!service.pid) throw new Error('collector service child did not expose a process id')
    serviceIdentity = await waitForProcessIdentity(service.pid, process.execPath)
    updateRunManifest(runContext, { status: 'waiting-for-service', service_process: serviceIdentity })
    await waitForPort(httpPortReservation.port, SERVICE_PORT_TIMEOUT_MS)

    const client = createCollectorClient(paths.serviceRoot, httpPortReservation.port)
    const health = await client.waitUntilReady()
    const mapPath = resolveMapPath(config, paths, scenarioTarget.kind)
    const map = readFileSync(mapPath)
    const summaries: ScenarioSummary[] = []
    for (const scenario of scenarios) {
      summaries.push(await executeScenario({
        scenario,
        map,
        mapPath,
        repoRoot: config.repoRoot,
        ...(config.room ? { room: config.room } : {}),
        skipTransitions: config.skipTransitions,
        collectOnly: config.collectOnly,
        dependencies: {
          simulate: async (request) => await client.simulate(request),
          writeTrace,
          compare: (options) => compareRealTrace({ repoRoot: config.repoRoot, ...options }),
        },
      }))
    }
    updateRunManifest(runContext, { status: 'completed' })
    return { health, scenarios: summaries }
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

function resolveMapPath(
  config: HarnessConfig,
  paths: ReturnType<typeof createHarnessPaths>,
  targetKind: 'playground' | 'external',
): string {
  if (config.mapPath) return config.mapPath
  return targetKind === 'playground'
    ? resolve(paths.playgroundModRoot, 'Maps', 'CelesteGymPlayground', 'Playground.bin')
    : resolve(paths.gameRoot, 'Content', 'Maps', config.mapFile)
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
