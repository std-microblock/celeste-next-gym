import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { near, semanticAssert } from '../../verify.js'
import { SPIKE_CLIMB_PART } from '../dashless-spike-parts.js'

export const mapParts = [SPIKE_CLIMB_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['3.8'],
  mapParts,
  name: 'spike-climb',
  recording: { primaryFor: ['3.8'], startFrame: 0, endFrame: 16 },
  initial: { pos: [59, 140], speed: [0, 0], facing: 'Right' },
  inputs: Array.from({ length: 16 }, (_, frame) => input({
    move_x: -1,
    jump_pressed: frame === 0,
    jump_held: frame < 12,
  })),
  verify: verifySpikeClimb,
})

function verifySpikeClimb(states: readonly E2EState[]): void {
  semanticAssert(states.every((state) => !state.dead), 'spike-climb', 'away-facing wall jump died on the directional spike wall')
  semanticAssert(near(states[1]?.speed[0], -130) && near(states[1]?.speed[1], -105), 'spike-climb', `launch was ${JSON.stringify(states[1]?.speed)}`)
  semanticAssert((states[8]?.pos[1] ?? 140) < 128, 'spike-climb', 'the spike-safe jump did not gain height')
}
