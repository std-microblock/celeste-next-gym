import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  mapParts,
  name: 'coyote-jump',
    initial: { pos: [42, 144], speed: [0, 0] },
    inputs: Array.from({ length: 12 }, (_, frame) => input({
      move_x: 1,
      jump_pressed: frame === 3,
      jump_held: frame >= 3 && frame < 9,
    })),
})


