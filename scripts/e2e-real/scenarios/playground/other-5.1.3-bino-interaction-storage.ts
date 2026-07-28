import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { field, semanticAssert } from '../../verify.js'
import { TECH_OTHER_5_1_3_BINO_INTERACTION_STORAGE } from '../lookout-parts.js'

export const mapParts = [TECH_OTHER_5_1_3_BINO_INTERACTION_STORAGE] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:lookout', 'feature:transition', 'debt:storage-transition'],
  techniqueIds: ['5.1.3'],
  mapParts,
  name: 'other-5.1.3-bino-interaction-storage',
  initial: { pos: [512, 496], speed: [0, 0], on_ground: true },
  inputs: Array.from({ length: 80 }, (_, frame) => input({ talk_pressed: frame === 0 })),
  verify(states) {
    semanticAssert(states.some((state) => field<boolean>(state, 'lookoutInteracting') === true), scenario.name,
      'baseline did not reach Lookout.interacting before the room-removal storage setup')
  },
})
