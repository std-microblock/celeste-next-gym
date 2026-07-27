import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['3.3'],
  recording: { primaryFor: ['3.3'], startFrame: 0, endFrame: 30 },
  mapParts,
  name: 'ceiling-pop',
    initial: { pos: [244, 78], speed: [0, 30] },
    inputs: Array.from({ length: 30 }, (_, frame) => input({
      move_x: frame === 18 ? 1 : 0,
      move_y: 1,
      grab_held: true,
      jump_pressed: frame === 18,
      jump_held: frame === 18,
    })),
})
