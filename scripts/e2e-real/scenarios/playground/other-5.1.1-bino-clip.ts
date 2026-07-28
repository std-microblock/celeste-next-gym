import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { field, semanticAssert } from '../../verify.js'
import { TECH_OTHER_5_1_1_BINO_CLIP } from '../lookout-parts.js'

export const mapParts = [TECH_OTHER_5_1_1_BINO_CLIP] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:lookout', 'feature:spinner'],
  techniqueIds: ['5.1.1'],
  mapParts,
  name: 'other-5.1.1-bino-clip',
  initial: { pos: [512, 496], speed: [0, 0], on_ground: true },
  inputs: Array.from({ length: 230 }, (_, frame) => input({
    talk_pressed: frame === 0,
    move_x: frame >= 55 && frame < 205 ? 1 : 0,
    jump_pressed: frame === 205,
    jump_held: frame === 205,
  })),
  verify(states) {
    const visible = states.map((state) => field<boolean>(state, 'crystalSpinnerVisible'))
    const collidable = states.map((state) => field<boolean>(state, 'crystalSpinnerCollidable'))
    semanticAssert(visible.some(Boolean), scenario.name, 'spinner never entered its real camera view')
    semanticAssert(visible.some((value, frame) => frame > 100 && value === false), scenario.name,
      '0.25-second off-screen interval never hid the spinner')
    semanticAssert(collidable.some((value, frame) => frame > 100 && value === false), scenario.name,
      'invisible spinner never disabled Collidable')
  },
})
