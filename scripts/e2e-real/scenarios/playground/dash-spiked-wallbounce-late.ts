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
  name: 'dash-spiked-wallbounce-late',
    initial: { pos: [396, 207], speed: [0, 0] },
    inputs: Array.from({ length: 7 }, (_, frame) => input({
      move_y: -1,
      jump_pressed: frame === 6,
      jump_held: frame === 6,
      dash_pressed: frame === 0,
    })),
    verify: verifyLateSpikedWallbounce,
})


function verifyLateSpikedWallbounce(states: readonly E2EState[]): void {
  semanticAssert(states.some((state) => state.dead), 'dash-spiked-wallbounce-late', 'one-frame-late input unexpectedly survived')
}

