import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  mapParts,
  name: 'jump',
    inputs: Array.from({ length: 45 }, (_, frame) => input({
      jump_pressed: frame === 0,
      jump_held: frame < 12,
    })),
})


