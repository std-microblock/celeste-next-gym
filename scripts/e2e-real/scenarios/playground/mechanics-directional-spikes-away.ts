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
  name: 'mechanics-directional-spikes-away',
    initial: { pos: [360, 496], speed: [0, -60] },
    inputs: Array.from({ length: 4 }, () => input()),
    verify: verifyDirectionalSpikesAway,
})


function verifyDirectionalSpikesAway(states: readonly E2EState[]): void {
  semanticAssert(states.every((state) => !state.dead), 'mechanics-directional-spikes-away', 'moving away from upward spikes must remain alive')
}

