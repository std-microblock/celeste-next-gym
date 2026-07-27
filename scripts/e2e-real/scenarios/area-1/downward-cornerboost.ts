import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['3.7.1'],
  recording: { primaryFor: ['3.7.1'], startFrame: 0, endFrame: 12 },
  mapParts,
  name: 'downward-cornerboost',
    initial: { pos: [138, 86], speed: [160, 30] },
    inputs: Array.from({ length: 12 }, (_, frame) => input({
      move_x: 1,
      jump_pressed: frame === 0,
      jump_held: frame < 8,
      grab_held: frame === 0,
    })),
})
