import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { near, semanticAssert } from '../../verify.js'
import { ENTITY_4_22_3_HOLDABLE_NEUTRAL_PART } from '../holdable-parts.js'

export const mapParts = [ENTITY_4_22_3_HOLDABLE_NEUTRAL_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['4.22.3'],
  mapParts,
  name: 'entity-4.22.3-holdable-neutral-jump',
  recording: { primaryFor: ['4.22.3'], startFrame: 0, endFrame: 40 },
  initial: { pos: [380, 420] },
  inputs: Array.from({ length: 40 }, (_, frame) => input({
    jump_pressed: frame === 23,
    jump_held: frame === 23,
    grab_held: true,
  })),
  verify: verifyHoldableNeutral,
})

function verifyHoldableNeutral(states: readonly E2EState[]): void {
  const firstPickup = states.findIndex((state) => state.state === 8 && state.holding_theo)
  const neutral = states.findIndex((state, frame) => frame > firstPickup
    && state.state === 0
    && state.holding_theo
    && near(state.speed[0], -130)
    && near(state.speed[1], -105))
  semanticAssert(firstPickup >= 0, scenario.name, 'Theo was not picked up')
  semanticAssert(neutral > firstPickup, scenario.name,
    `holding Theo did not force the grabbed wall jump onto the normal-neutral branch: ${neutral}`)
}
