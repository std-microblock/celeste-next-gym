import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  mapParts,
  name: 'wavedash',
    initial: { pos: [70, 112], speed: [0, 0] },
    inputs: Array.from({ length: 18 }, (_, frame) => input({
      move_x: 1,
      move_y: frame <= 10 ? 1 : 0,
      jump_pressed: frame === 10,
      jump_held: frame >= 10 && frame < 16,
      dash_pressed: frame === 0,
    })),
})


