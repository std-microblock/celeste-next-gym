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
  name: 'up-diagonal-demo',
    inputs: Array.from({ length: 12 }, (_, frame) => input({
      move_x: 1,
      move_y: -1,
      crouch_dash_pressed: frame === 0,
    })),
})
