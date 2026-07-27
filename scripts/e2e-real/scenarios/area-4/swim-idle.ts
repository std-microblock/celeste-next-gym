import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_4_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_4_TARGET,
  status: 'active',
  tags: [],
  mapParts,
  name: 'swim-idle',
    initial: { pos: [1512, -248], speed: [0, 0] },
    inputs: Array.from({ length: 30 }, () => input()),
})


