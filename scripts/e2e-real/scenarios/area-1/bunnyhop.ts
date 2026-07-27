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
  name: 'bunnyhop',
    initial: { pos: [19, 135], speed: [160, 100] },
    inputs: Array.from({ length: 18 }, (_, frame) => input({
      move_x: 1,
      jump_pressed: frame === 3,
      jump_held: frame >= 3 && frame < 11,
    })),
})
