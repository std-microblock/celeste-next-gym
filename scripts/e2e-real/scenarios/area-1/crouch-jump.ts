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
  name: 'crouch-jump',
    initial: { pos: [42, 144], speed: [0, 0] },
    inputs: Array.from({ length: 40 }, (_, frame) => input({
      move_y: frame <= 1 ? 1 : 0,
      jump_pressed: frame === 1,
      jump_held: frame >= 1 && frame < 10,
    })),
})
