import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  mapParts,
  name: 'wallboost',
    initial: { pos: [140, 112], speed: [0, 30] },
    inputs: Array.from({ length: 12 }, (_, frame) => input({
      move_x: frame === 4 ? -1 : 0,
      grab_held: frame <= 3,
      jump_pressed: frame === 3,
      jump_held: frame >= 3 && frame < 10,
    })),
})


