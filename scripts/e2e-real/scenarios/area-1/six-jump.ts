import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  mapParts,
  name: 'six-jump',
    initial: { pos: [139, 86], speed: [90, -30] },
    inputs: Array.from({ length: 40 }, (_, frame) => input({
      move_x: 1,
      jump_pressed: frame === 0,
      jump_held: frame < 13,
      grab_held: frame === 0,
    })),
})


