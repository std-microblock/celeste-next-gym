import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { semanticAssert } from '../../verify.js'
import { TECH_OTHER_5_2_BUBSDROP } from '../bubs-parts.js'

export const mapParts = [TECH_OTHER_5_2_BUBSDROP] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET, status: 'candidate',
  tags: ['feature:transition', 'feature:jump-thru', 'debt:respawn-point'],
  techniqueIds: ['5.2'], mapParts, name: 'other-5.2-bubsdrop',
  initial: { pos: [452, 4], speed: [0, -160], dashes: 0, stamina: 20 },
  inputs: Array.from({ length: 100 }, (_, frame) => input({ grab_held: frame > 45 && frame < 65, jump_pressed: frame === 58, jump_held: frame === 58 })),
  verify(states) {
    semanticAssert(states.some((state) => state.speed[1] === -105), scenario.name,
      'upward transition did not install BeforeUpTransition auto-jump')
  },
})
