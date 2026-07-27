import type { HarnessConfig } from '../config.js'
import { selectScenarios, type ScenarioRegistry } from '../registry.js'
import type { ScenarioDefinition, ScenarioTarget, TargetId } from '../types.js'
import type { TechniqueCatalog } from './techniques.js'

export interface RecordingScenarioPlan {
  readonly scenario: ScenarioDefinition
  readonly techniqueIds: readonly string[]
}

export interface RecordingTargetGroup {
  readonly target: ScenarioTarget
  readonly scenarios: readonly RecordingScenarioPlan[]
}

export interface RecordingPlan {
  readonly groups: readonly RecordingTargetGroup[]
  readonly scenarioCount: number
  readonly techniqueCount: number
}

const TARGET_ORDER: readonly TargetId[] = ['playground', 'area-1', 'area-2', 'area-4']

export function createRecordingPlan(
  config: HarnessConfig,
  registry: ScenarioRegistry,
  catalog: TechniqueCatalog,
): RecordingPlan {
  if (!config.recording) throw new Error('recording plan requires an explicit recording mode')
  const planned = new Map<string, { scenario: ScenarioDefinition; techniqueIds: Set<string> }>()

  if (config.recording.kind === 'scenario') {
    const name = [...config.explicitScenarioNames][0]
    if (!name) throw new Error('--record requires exactly one explicit --scenario')
    const [scenario] = selectScenarios(registry, {
      names: new Set([name]),
      ...(config.targetExplicit && config.target ? { target: config.target } : {}),
      includeCandidates: false,
      disabledTags: config.disabledTags,
    })
    if (!scenario || scenario.status !== 'active') throw new Error(`${name}: candidate scenarios cannot be recorded`)
    planned.set(scenario.name, {
      scenario,
      techniqueIds: new Set(scenario.recording?.primaryFor ?? []),
    })
  } else {
    const requestedTechniques = config.recording.kind === 'all'
      ? [...catalog.implementedIds]
      : [...config.recording.techniqueIds]
    for (const techniqueId of requestedTechniques) {
      const technique = catalog.byId.get(techniqueId)
      if (!technique) throw new Error(`unknown technique id: ${techniqueId}`)
      if (technique.status !== 'implemented') throw new Error(`technique ${techniqueId} is unimplemented and cannot be recorded`)
      const primaries = registry.scenarios.filter((scenario) => scenario.recording?.primaryFor.includes(techniqueId))
      if (primaries.length !== 1) throw new Error(`implemented technique ${techniqueId} requires exactly one primary scenario, found ${primaries.length}`)
      const scenario = primaries[0]
      if (!scenario || scenario.status !== 'active') throw new Error(`implemented technique ${techniqueId} primary is not active`)
      if (config.targetExplicit && config.target && scenario.target.id !== config.target) {
        throw new Error(`technique ${techniqueId} primary target ${scenario.target.id} conflicts with ${config.target}`)
      }
      if ([...config.disabledTags].some((tag) => scenario.tags.includes(tag))) {
        throw new Error(`technique ${techniqueId} primary ${scenario.name} is disabled by a feature filter`)
      }
      const item = planned.get(scenario.name) ?? { scenario, techniqueIds: new Set<string>() }
      item.techniqueIds.add(techniqueId)
      planned.set(scenario.name, item)
    }
  }

  const groups = TARGET_ORDER.flatMap((targetId) => {
    const scenarios = [...planned.values()]
      .filter((item) => item.scenario.target.id === targetId)
      .sort((left, right) => left.scenario.name.localeCompare(right.scenario.name))
      .map((item) => Object.freeze({
        scenario: item.scenario,
        techniqueIds: Object.freeze([...item.techniqueIds].sort()),
      }))
    if (scenarios.length === 0) return []
    return [Object.freeze({ target: scenarios[0]!.scenario.target, scenarios: Object.freeze(scenarios) })]
  })
  return Object.freeze({
    groups: Object.freeze(groups),
    scenarioCount: planned.size,
    techniqueCount: new Set([...planned.values()].flatMap((item) => [...item.techniqueIds])).size,
  })
}
