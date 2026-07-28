import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { field, semanticAssert } from '../../verify.js'
import { TECH_OTHER_5_2_BUBSDROP } from '../bubs-parts.js'

export const mapParts = [TECH_OTHER_5_2_BUBSDROP] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET, status: 'candidate',
  tags: ['feature:transition', 'feature:jump-thru', 'feature:wall-jump', 'debt:respawn-point'],
  techniqueIds: ['5.2'], mapParts, name: 'other-5.2-bubsdrop',
  initial: { pos: [452, 4], speed: [0, -160], dashes: 0, stamina: 20 },
  inputs: Array.from({ length: 140 }, (_, frame) => input({
    jump_pressed: frame === 41,
    jump_held: frame >= 41 && frame < 51,
  })),
  verify(states) {
    semanticAssert(states.some((state) => state.speed[1] === -105), scenario.name,
      'upward transition did not install BeforeUpTransition auto-jump')
    semanticAssert(states.some((state) => state.speed[0] === -130 && state.speed[1] === -105), scenario.name,
      'wall jump did not replace the auto-jump trajectory')
    semanticAssert(states.some((state, frame) => frame > 41 && state.pos[1] > 0), scenario.name,
      'wall-jump route did not miss the upper JumpThru and return to the old room')
    semanticAssert(states.some((state, frame) => {
      const respawn = field<readonly number[]>(state, 'sessionRespawnPoint')
      return frame > 100 && respawn?.[0] === 440 && respawn[1] === 496
    }), scenario.name, 'return transition did not choose the nearer old-room spawn [440,496]')
  },
})
