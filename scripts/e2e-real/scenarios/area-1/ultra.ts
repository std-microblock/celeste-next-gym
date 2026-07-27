import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  mapParts,
  name: 'ultra',
    initial: { pos: [150, 45], speed: [0, 0] },
    inputs: Array.from({ length: 24 }, (_, frame) => input({
      move_x: 1,
      move_y: frame <= 4 ? 1 : 0,
      dash_pressed: frame === 0,
    })),
})


