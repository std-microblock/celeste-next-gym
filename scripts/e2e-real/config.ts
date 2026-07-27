import { resolve } from 'node:path'

import { PLAYGROUND_SID } from './constants.js'
import { TARGETS, targetForArea } from './targets.js'
import type { TargetId } from './types.js'

export interface HarnessConfig {
  readonly repoRoot: string
  readonly showGameWindow: boolean
  readonly skipTransitions: boolean
  readonly collectOnly: boolean
  readonly expectedGitBranch?: string
  readonly expectedGitHead?: string
  readonly requestedScenarios: ReadonlySet<string>
  readonly target?: TargetId
  readonly areaId: number
  readonly areaSid?: string
  readonly mapFile: string
  readonly mapPath?: string
  readonly room?: string
  readonly steamRoots: readonly string[]
  readonly includeCandidates: boolean
  readonly listOnly: boolean
  readonly fixtureOutput?: string
  readonly disabledTags: ReadonlySet<string>
}

const FEATURE_ENV = Object.freeze({
  E2E_PLAYGROUND_SWIM: 'feature:swim',
  E2E_PLAYGROUND_BOOSTER: 'feature:booster',
  E2E_PLAYGROUND_WIND: 'feature:wind',
  E2E_PLAYGROUND_STAR_FLY: 'feature:star-fly',
  E2E_PLAYGROUND_LAUNCH: 'feature:launch',
  E2E_PLAYGROUND_BUMPER: 'feature:bumper',
  E2E_PLAYGROUND_BADELINE_BOOST: 'feature:badeline-boost',
  E2E_PLAYGROUND_MISC_STATES: 'feature:misc-states',
  E2E_PLAYGROUND_ZIP_MOVER: 'feature:zip-mover',
  E2E_PLAYGROUND_BOUNCE_BLOCK: 'feature:bounce-block',
} as const)

const DEFAULT_MAPS = new Map<number, string>([
  [1, '1-ForsakenCity.bin'],
  [2, '2-OldSite.bin'],
  [4, '4-GoldenRidge.bin'],
])

export function parseConfig(
  argv: readonly string[],
  env: NodeJS.ProcessEnv,
  repoRoot: string,
): HarnessConfig {
  let listOnly = false
  let includeCandidates = false
  let cliTarget: TargetId | undefined
  let fixtureOutput: string | undefined
  const cliNames = new Set<string>()
  for (let index = 0; index < argv.length; index++) {
    const argument = argv[index]
    if (argument === '--list') listOnly = true
    else if (argument === '--include-candidates') includeCandidates = true
    else if (argument === '--scenario') cliNames.add(requireValue(argv, ++index, argument))
    else if (argument === '--target') cliTarget = parseTarget(requireValue(argv, ++index, argument))
    else if (argument === '--fixture-output') fixtureOutput = resolve(repoRoot, requireValue(argv, ++index, argument))
    else throw new Error(`unknown E2E harness argument: ${argument}`)
  }

  const areaId = parseAreaId(env.E2E_AREA_ID ?? '1')
  const areaSid = trimmed(env.E2E_AREA_SID)
  const envTarget = targetForArea(areaId, areaSid)?.id
  if (envTarget === undefined) throw new Error(`unknown E2E target for area ${areaId}${areaSid ? ` (${areaSid})` : ''}`)
  if (cliTarget && (env.E2E_AREA_ID !== undefined || env.E2E_AREA_SID !== undefined) && envTarget !== cliTarget) {
    throw new Error(`--target ${cliTarget} conflicts with E2E_AREA_ID/E2E_AREA_SID (${envTarget ?? 'unknown'})`)
  }
  const target = cliTarget ?? envTarget
  const effectiveAreaId = target ? TARGETS[target].areaId : areaId
  const requestedScenarios = cliNames.size > 0 ? cliNames : commaSet(env.E2E_SCENARIOS)
  const disabledTags = new Set<string>()
  for (const [name, tag] of Object.entries(FEATURE_ENV)) if (env[name] === '0') disabledTags.add(tag)

  const showGameWindow = env.E2E_SHOW_WINDOWS === '1'
  const expectedGitBranch = trimmed(env.E2E_EXPECT_GIT_BRANCH)
  const expectedGitHead = trimmed(env.E2E_EXPECT_GIT_HEAD)
  const mapPath = trimmed(env.E2E_MAP_PATH)
  const room = trimmed(env.E2E_ROOM)
  return Object.freeze({
    repoRoot: resolve(repoRoot),
    showGameWindow,
    skipTransitions: env.E2E_SKIP_TRANSITIONS === '1' || showGameWindow,
    collectOnly: env.E2E_COLLECT_ONLY === '1',
    ...(expectedGitBranch ? { expectedGitBranch } : {}),
    ...(expectedGitHead ? { expectedGitHead } : {}),
    requestedScenarios,
    ...(target ? { target } : {}),
    areaId: effectiveAreaId,
    ...(target === 'playground' ? { areaSid: PLAYGROUND_SID } : areaSid ? { areaSid } : {}),
    mapFile: env.E2E_MAP_FILE ?? DEFAULT_MAPS.get(effectiveAreaId) ?? `${effectiveAreaId}-Unknown.bin`,
    ...(mapPath ? { mapPath: resolve(repoRoot, mapPath) } : {}),
    ...(room ? { room } : {}),
    steamRoots: splitPaths(env.E2E_STEAM_CELESTE_ROOTS),
    includeCandidates,
    listOnly,
    ...(fixtureOutput ? { fixtureOutput } : {}),
    disabledTags,
  })
}

function requireValue(argv: readonly string[], index: number, flag: string): string {
  const value = argv[index]
  if (!value || value.startsWith('--')) throw new Error(`${flag} requires a value`)
  return value
}

function parseTarget(value: string): TargetId {
  if (value === 'playground' || value === 'area-1' || value === 'area-2' || value === 'area-4') return value
  throw new Error(`unknown E2E target: ${value}`)
}

function parseAreaId(value: string): number {
  const areaId = Number.parseInt(value, 10)
  if (!Number.isSafeInteger(areaId) || areaId < 0 || String(areaId) !== value.trim()) {
    throw new Error('E2E_AREA_ID must be a non-negative integer')
  }
  return areaId
}

function trimmed(value: string | undefined): string | undefined {
  return value?.trim() || undefined
}

function commaSet(value: string | undefined): ReadonlySet<string> {
  return new Set((value ?? '').split(',').map((name) => name.trim()).filter(Boolean))
}

function splitPaths(value: string | undefined): readonly string[] {
  return Object.freeze((value ?? '').split(process.platform === 'win32' ? ';' : ':').map((path) => path.trim()).filter(Boolean))
}
