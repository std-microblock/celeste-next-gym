import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  mapParts,
  name: 'five-jump',
    initial: { pos: [44, 156], speed: [0, 0], state: 'Climb', facing: 'Left' },
    inputs: Array.from({ length: 48 }, (_, frame) => input({
      move_x: frame >= 6 ? 1 : 0,
      jump_pressed: frame === 0 || frame === 5,
      jump_held: frame <= 17,
      grab_held: frame === 0 || frame === 5,
    })),
})


