import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { near, semanticAssert } from '../../verify.js'
import { ENTITY_4_22_2_HOLDABLE_CLIMB_PART } from '../holdable-parts.js'

export const mapParts = [ENTITY_4_22_2_HOLDABLE_CLIMB_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: [],
  techniqueIds: ['4.22.2'],
  mapParts,
  name: 'entity-4.22.2-holdable-climb',
  initial: { pos: [220, 420] },
  inputs: Array.from({ length: 50 }, (_, frame) => input({
    move_y: frame === 23 ? 1 : 0,
    jump_pressed: frame === 25,
    jump_held: frame === 25,
    grab_held: frame <= 22 || frame >= 24,
  })),
  verify: verifyHoldableClimb,
})

function verifyHoldableClimb(states: readonly E2EState[]): void {
  const firstPickup = states.findIndex((state) => state.state === 8 && state.holding_theo)
  const released = states.findIndex((state, frame) => frame > firstPickup && !state.holding_theo)
  const climb = states.findIndex((state, frame) => frame > released && state.state === 1 && !state.holding_theo)
  const climbJump = states.findIndex((state, frame) => frame > climb && state.state === 0 && near(state.speed[1], -105))
  const regrabbed = states.findIndex((state, frame) => frame > climbJump && state.state === 8 && state.holding_theo)
  semanticAssert(firstPickup >= 0 && released > firstPickup, scenario.name, `pickup=${firstPickup}, release=${released}`)
  semanticAssert(climb > released && climbJump > climb, scenario.name,
    `drop did not enable climb jump: climb=${climb}, jump=${climbJump}`)
  semanticAssert(regrabbed > climbJump, scenario.name, `Theo was not recovered after climb jump: ${regrabbed}`)
}
