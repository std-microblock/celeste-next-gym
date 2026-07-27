import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['2.10'],
  recording: { primaryFor: ['2.10'], startFrame: 0, endFrame: 14 },
  mapParts,
  name: 'wallbounce',
    initial: { pos: [140, 112], speed: [0, 0] },
    inputs: Array.from({ length: 14 }, (_, frame) => input({
      move_y: -1,
      jump_pressed: frame === 5,
      jump_held: frame >= 5 && frame < 12,
      dash_pressed: frame === 0,
    })),
})
