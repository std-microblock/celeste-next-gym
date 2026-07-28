import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { field, semanticAssert } from '../../verify.js'
import { TECH_OTHER_5_1_2_BINO_CONTROL_STORAGE } from '../lookout-parts.js'

export const mapParts = [TECH_OTHER_5_1_2_BINO_CONTROL_STORAGE] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:lookout', 'debt:storage-interrupt'],
  techniqueIds: ['5.1.2'],
  mapParts,
  name: 'other-5.1.2-bino-control-storage',
  initial: { pos: [512, 496], speed: [0, 0], on_ground: true },
  inputs: Array.from({ length: 120 }, (_, frame) => input({ talk_pressed: frame === 0, move_x: frame >= 55 ? 1 : 0 })),
  verify(states) {
    semanticAssert(states.some((state) => field<boolean>(state, 'lookoutInteracting') === true), scenario.name,
      'baseline Lookout interaction did not become active')
    semanticAssert(states.some((state) => (field<readonly number[]>(state, 'levelCamera')?.[0] ?? 0) > 360), scenario.name,
      'baseline Lookout camera control did not move')
  },
})
