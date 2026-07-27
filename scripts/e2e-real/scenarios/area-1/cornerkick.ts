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
  name: 'cornerkick',
    initial: { pos: [242, 90], speed: [0, -30] },
    inputs: Array.from({ length: 12 }, (_, frame) => input({
      move_x: 1,
      jump_pressed: frame === 0,
      jump_held: frame < 6,
    })),
})
