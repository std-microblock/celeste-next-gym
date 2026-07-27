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
  name: 'wallboost-neutral',
    initial: { pos: [140, 112], speed: [0, 30] },
    inputs: Array.from({ length: 60 }, (_, frame) => input({
      move_x: frame === 4 || frame === 31
        ? -1
        : (frame >= 5 && frame <= 29) || frame >= 32
          ? 1
          : 0,
      grab_held: frame <= 3 || frame >= 20,
      jump_pressed: frame === 3 || frame === 30,
      jump_held: (frame >= 3 && frame < 13) || (frame >= 30 && frame < 40),
    })),
})
