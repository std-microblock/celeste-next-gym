import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { near, semanticAssert } from '../../verify.js'
import { SPIKE_CLIP_PART } from '../dashless-spike-parts.js'

export const mapParts = [SPIKE_CLIP_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: [],
  techniqueIds: ['3.9'],
  mapParts,
  name: 'spike-clip',
  initial: { pos: [92, 103], speed: [0, 240] },
  inputs: Array.from({ length: 6 }, () => input({ move_y: 1 })),
  verify: verifySpikeClip,
})

function verifySpikeClip(states: readonly E2EState[]): void {
  semanticAssert(states.every((state) => !state.dead), 'spike-clip', 'high-speed fall was caught by the unsupported spike strip')
  semanticAssert(near(states[1]?.pos[1], 107) && (states[1]?.speed[1] ?? 0) > 220, 'spike-clip', `first crossing was pos=${states[1]?.pos[1]} speed=${states[1]?.speed[1]}`)
  semanticAssert((states[1]?.pos[1] ?? 0) - 2 > 103, 'spike-clip', 'hurtbox bottom did not cross below the three-pixel spike collider')
}
