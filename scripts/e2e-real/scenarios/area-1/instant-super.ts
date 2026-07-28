import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: [],
  mapParts,
  name: 'instant-super',
  inputs: Array.from({ length: 8 }, (_, frame) => input({
    dash_pressed: frame === 0,
    jump_pressed: frame === 1,
    jump_held: frame >= 1,
  })),
  verify(states) {
    const launch = states.find((state) => state.state === 0 && state.speed[0] === 260 && state.speed[1] === -105)
    if (!launch) throw new Error(`instant-super: missing 260/-105 launch: ${JSON.stringify(states)}`)
  },
})
