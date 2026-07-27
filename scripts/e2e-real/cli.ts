import { mkdirSync, writeFileSync } from 'node:fs'
import { dirname } from 'node:path'

import { parseConfig } from './config.js'
import { assembleFixturePackage } from './map-parts.js'
import { buildRegistry, selectScenarios } from './registry.js'
import { scenarios } from './scenarios/index.js'
import { runHarness } from './runtime/runner.js'
import type { HarnessSummary } from './runtime/runner.js'
import type { HarnessConfig } from './config.js'
import type { ScenarioDefinition } from './types.js'

export interface MainDependencies {
  run(config: HarnessConfig, selected: readonly ScenarioDefinition[]): Promise<HarnessSummary>
}

export async function main(
  argv: readonly string[],
  env: NodeJS.ProcessEnv,
  repoRoot: string,
  dependencies: MainDependencies = { run: runHarness },
): Promise<void> {
  const config = parseConfig(argv, env, repoRoot)
  const registry = buildRegistry(scenarios)
  const selected = selectScenarios(registry, {
    ...(config.requestedScenarios.size > 0 ? { names: config.requestedScenarios } : {}),
    ...(config.target ? { target: config.target } : {}),
    includeCandidates: config.includeCandidates,
    disabledTags: config.disabledTags,
  })

  if (config.listOnly) {
    console.log(JSON.stringify({ counts: registry.counts, scenarios: selected.map(describeScenario) }, null, 2))
    return
  }
  if (config.fixtureOutput) {
    const requestedParts = [...new Map(selected.flatMap((scenario) => scenario.mapParts).map((part) => [part.id, part])).values()]
    const fixture = assembleFixturePackage(requestedParts, registry.mapParts)
    mkdirSync(dirname(config.fixtureOutput), { recursive: true })
    writeFileSync(config.fixtureOutput, `${JSON.stringify(fixture, null, 2)}\n`, 'utf8')
    console.log(JSON.stringify({ fixture: config.fixtureOutput, scenarios: selected.length, parts: requestedParts.length }, null, 2))
    return
  }

  const summary = await dependencies.run(config, selected)
  console.log(JSON.stringify(summary, null, 2))
}

function describeScenario(scenario: (typeof scenarios)[number]): Record<string, unknown> {
  return {
    name: scenario.name,
    target: scenario.target.id,
    status: scenario.status,
    tags: scenario.tags,
    techniqueIds: scenario.techniqueIds,
    mapParts: scenario.mapParts.map((part: { readonly id: string }) => part.id),
  }
}
