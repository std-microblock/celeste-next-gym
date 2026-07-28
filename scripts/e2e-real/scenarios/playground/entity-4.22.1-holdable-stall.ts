import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { semanticAssert } from '../../verify.js'
import { ENTITY_4_22_1_HOLDABLE_STALL_PART } from '../glider-parts.js'

export const mapParts = [ENTITY_4_22_1_HOLDABLE_STALL_PART] as const
export const scenario = defineScenario({
  target: PLAYGROUND_TARGET, status: 'active', tags: ['feature:glider'], techniqueIds: ['4.22.1'], mapParts,
  name: 'entity-4.22.1-holdable-stall', initial: { pos: [80, 320], speed: [0, -20] },
  recording: { primaryFor: ['4.22.1'], startFrame: 0, endFrame: 96 },
  inputs: Array.from({ length: 96 }, (_, frame) => input({
    move_y: frame === 23 || frame === 65 ? 1 : -1,
    grab_held: frame <= 22 || (frame >= 24 && frame <= 64) || frame >= 66,
  })),
  verify: verifyHoldableStall,
})

function verifyHoldableStall(states: readonly E2EState[]): void {
  const pickups = states.flatMap((state, frame) => state.state === 8 && state.holding_glider ? [frame] : [])
  const distinct = pickups.filter((frame, index) => index === 0 || frame > (pickups[index - 1] ?? frame) + 1)
  semanticAssert(distinct.length >= 2, scenario.name, `pickup phases=${distinct.join(',')}`)
  const first = distinct[0] ?? Number.MAX_SAFE_INTEGER
  semanticAssert(states.some((state, frame) => frame > first && !state.holding_glider), scenario.name, 'neutral drop was not observed')
}
