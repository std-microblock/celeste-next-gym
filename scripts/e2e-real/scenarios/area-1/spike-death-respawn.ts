import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  mapParts,
  name: 'spike-death-respawn',
    initial: { pos: [60, 150], speed: [0, 90] },
    inputs: Array.from({ length: 120 }, () => input()),
})


