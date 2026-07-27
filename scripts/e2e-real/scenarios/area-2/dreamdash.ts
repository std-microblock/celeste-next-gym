import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_2_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_2_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: [],
  mapParts,
  name: 'dreamdash',
    initial: { pos: [776, -50], speed: [0, 0] },
    inputs: Array.from({ length: 40 }, (_, frame) => input({
      move_x: 1,
      dash_pressed: frame === 0,
    })),
})
