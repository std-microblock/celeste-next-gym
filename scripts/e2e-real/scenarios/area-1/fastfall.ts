import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['1.8'],
  recording: { primaryFor: ['1.8'], startFrame: 0, endFrame: 24 },
  mapParts,
  name: 'fastfall',
    initial: { pos: [120, 60], speed: [0, 160] },
    inputs: Array.from({ length: 24 }, () => input({ move_y: 1 })),
})
