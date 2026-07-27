import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_SPIKES } from '../common-parts.js'
import type { E2EState } from '../../types.js'
import { field, near, pickCore, semanticAssert } from '../../verify.js'

export const mapParts = [PLAYGROUND_SPIKES] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  mapParts,
  name: 'mechanics-directional-spikes-into',
    initial: { pos: [360, 496], speed: [0, 60] },
    inputs: Array.from({ length: 4 }, () => input()),
    verify: verifyDirectionalSpikesInto,
})


function verifyDirectionalSpikesInto(states: readonly E2EState[]): void {
  semanticAssert(states[1]?.dead === true, 'mechanics-directional-spikes-into', 'moving into upward spikes must die on frame 1')
}

