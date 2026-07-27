import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { near, semanticAssert } from '../../verify.js'
import { NARROW_SPIKED_CLIMB_PART } from '../dashless-spike-parts.js'

export const mapParts = [NARROW_SPIKED_CLIMB_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: [],
  techniqueIds: ['3.8.1'],
  mapParts,
  name: 'narrow-spiked-climb',
  initial: { pos: [59, 152], speed: [0, 0], facing: 'Right' },
  inputs: Array.from({ length: 5 }, (_, frame) => input({
    jump_pressed: frame === 0 || frame === 3,
    jump_held: true,
  })),
  verify: verifyNarrowSpikedClimb,
})

function verifyNarrowSpikedClimb(states: readonly E2EState[]): void {
  semanticAssert(states.every((state) => !state.dead), 'narrow-spiked-climb', 'alternating wall jumps died in the narrow spiked shaft')
  semanticAssert(near(states[1]?.speed[0], -130) && near(states[4]?.speed[0], 130), 'narrow-spiked-climb', `away speeds were ${states[1]?.speed[0]} and ${states[4]?.speed[0]}`)
  semanticAssert((states[5]?.pos[1] ?? 152) < 145, 'narrow-spiked-climb', 'the alternating jumps did not climb the shaft')
}
