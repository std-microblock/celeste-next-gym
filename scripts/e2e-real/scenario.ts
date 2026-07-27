import type { ScenarioDefinition } from './types.js'

export function defineScenario<const T extends ScenarioDefinition>(scenario: T): T {
  if (!/^[a-z0-9][a-z0-9.-]*$/.test(scenario.name)) throw new Error(`invalid scenario name: ${scenario.name}`)
  if (scenario.inputs.length === 0) throw new Error(`${scenario.name}: inputs must not be empty`)
  for (const [frame, state] of scenario.inputs.entries()) {
    if (![-1, 0, 1].includes(state.move_x) || ![-1, 0, 1].includes(state.move_y)) {
      throw new Error(`${scenario.name}: invalid axis at frame ${frame}`)
    }
  }
  return Object.freeze(scenario)
}
