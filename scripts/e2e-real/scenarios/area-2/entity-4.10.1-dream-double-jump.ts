import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_2_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_2_TARGET,
  status: 'active',
  tags: [],
  mapParts,
  name: 'entity-4.10.1-dream-double-jump',
    initial: { pos: [776, -50], speed: [0, 0] },
    inputs: Array.from({ length: 36 }, (_, frame) => input({
      move_x: 1,
      jump_pressed: frame === 15 || frame === 17,
      jump_held: frame >= 15 && frame < 29,
      dash_pressed: frame === 0,
    })),
})


