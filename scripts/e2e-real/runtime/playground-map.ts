import { createHash } from 'node:crypto'
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

import { assembleFixturePackage } from '../map-parts.js'
import { COMMON_MAP_PARTS } from '../scenarios/common-parts.js'
import type { FixturePackage, ScenarioDefinition } from '../types.js'
import { runCommand } from './commands.js'
import type { HarnessPaths } from './prepare-mods.js'

export interface ScenarioFixture {
  readonly fixture: FixturePackage
  readonly json: string
  readonly hash: string
}

export interface MaterializedPlaygroundMap extends ScenarioFixture {
  readonly fixturePath: string
  readonly mapPath: string
  readonly modRoot: string
}

export interface ScenarioLifecycleGroup {
  readonly key: string
  readonly scenarios: readonly ScenarioDefinition[]
}

export function scenarioFixture(scenario: ScenarioDefinition): ScenarioFixture {
  if (scenario.target.kind !== 'playground') throw new Error(`${scenario.name}: only Playground scenarios have generated fixtures`)
  const fixture = assembleFixturePackage(scenario.mapParts, COMMON_MAP_PARTS)
  const json = `${JSON.stringify(fixture, null, 2)}\n`
  const hash = createHash('sha256').update(json).digest('hex')
  return Object.freeze({ fixture, json, hash })
}

export function scenarioLifecycleKey(scenario: ScenarioDefinition): string {
  return scenario.target.kind === 'playground'
    ? `${scenario.target.id}:${scenarioFixture(scenario).hash}`
    : scenario.target.id
}

export function assertNoPlaygroundMapOverride(scenario: ScenarioDefinition, mapPath?: string): void {
  if (scenario.target.kind === 'playground' && mapPath) {
    throw new Error('E2E_MAP_PATH cannot override generated per-scenario Playground maps')
  }
}

export function groupScenariosByLifecycle(scenarios: readonly ScenarioDefinition[]): readonly ScenarioLifecycleGroup[] {
  const groups = new Map<string, ScenarioDefinition[]>()
  for (const scenario of scenarios) {
    const key = scenarioLifecycleKey(scenario)
    const group = groups.get(key) ?? []
    group.push(scenario)
    groups.set(key, group)
  }
  return Object.freeze([...groups.entries()].map(([key, group]) => Object.freeze({
    key,
    scenarios: Object.freeze(group),
  })))
}

export function materializePlaygroundMap(options: {
  readonly repoRoot: string
  readonly runRoot: string
  readonly paths: HarnessPaths
  readonly scenario: ScenarioDefinition
  readonly compile?: (fixturePath: string, mapPath: string) => void
}): MaterializedPlaygroundMap {
  const generated = scenarioFixture(options.scenario)
  const root = resolve(options.runRoot, 'playground-map', generated.hash)
  const fixturePath = resolve(root, 'fixture.json')
  const mapPath = resolve(root, 'Playground.bin')
  mkdirSync(root, { recursive: true })
  writeFileSync(fixturePath, generated.json, 'utf8')
  const compile = options.compile ?? ((input, output) => runCommand('cargo', [
    'run', '-q', '-p', 'celeste-physics', '--example', 'compile_map_fixture', '--', input, output,
  ], options.repoRoot))
  compile(fixturePath, mapPath)
  const modRoot = stagePlaygroundMod(options.paths, root, mapPath)
  if (!readFileSync(resolve(modRoot, 'Maps', 'CelesteGymPlayground', 'Playground.bin')).equals(readFileSync(mapPath))) {
    throw new Error('staged Playground mod map differs from the scenario map used for Rust comparison')
  }
  return Object.freeze({ ...generated, fixturePath, mapPath, modRoot })
}

export function stagePlaygroundMod(paths: HarnessPaths, root: string, mapPath: string): string {
  const modRoot = resolve(root, 'mod')
  const stagedMap = resolve(modRoot, 'Maps', 'CelesteGymPlayground', 'Playground.bin')
  mkdirSync(resolve(stagedMap, '..'), { recursive: true })
  copyFileSync(resolve(paths.playgroundModRoot, 'everest.yaml'), resolve(modRoot, 'everest.yaml'))
  copyFileSync(mapPath, stagedMap)
  return modRoot
}
