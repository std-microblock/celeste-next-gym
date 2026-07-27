import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['3.11'],
  recording: { primaryFor: ['3.11'], startFrame: 0, endFrame: 30 },
  mapParts,
  name: 'stamina-cancel',
    initial: { pos: [140, 112], speed: [0, 30] },
    inputs: Array.from({ length: 30 }, (_, frame) => input({
      move_y: -1,
      grab_held: frame < 8 || frame >= 11,
    })),
})
