import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  mapParts,
  name: 'double-cornerboost',
    initial: { pos: [120, 152], speed: [0, 0], on_ground: true },
    inputs: Array.from({ length: 90 }, (_, frame) => input({
      move_x: frame <= 20 || frame >= 78
        ? 1
        : frame >= 75 && frame <= 77
          ? -1
          : 0,
      move_y: frame >= 21 && frame <= 74 ? -1 : 0,
      jump_pressed: frame === 0 || frame === 79 || frame === 80,
      jump_held: frame < 12 || frame === 79 || frame === 80,
      grab_held: frame <= 74 || frame === 79 || frame === 80,
    })),
})


