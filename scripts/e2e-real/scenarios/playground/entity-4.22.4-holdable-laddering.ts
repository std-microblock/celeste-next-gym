import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { semanticAssert } from '../../verify.js'
import { ENTITY_4_22_4_HOLDABLE_LADDER_PART } from '../glider-parts.js'

export const mapParts = [ENTITY_4_22_4_HOLDABLE_LADDER_PART] as const
export const scenario = defineScenario({
  target: PLAYGROUND_TARGET, status: 'candidate', tags: ['feature:glider'], techniqueIds: ['4.22.4'], mapParts,
  name: 'entity-4.22.4-holdable-laddering', initial: { pos: [96, 400], speed: [0, -30] },
  inputs: Array.from({ length: 150 }, (_, frame) => input({
    move_y: frame === 23 || frame === 65 || frame === 101 ? 1 : -1,
    grab_held: ![23, 65, 101].includes(frame),
  })),
  verify: verifyLaddering,
})

function verifyLaddering(states: readonly E2EState[]): void {
  const pickupStarts = states.flatMap((state, frame) => state.state === 8 && states[frame - 1]?.state !== 8 ? [frame] : [])
  const highest = Math.min(...states.map((state) => state.pos[1]))
  semanticAssert(pickupStarts.length >= 3, scenario.name, `pickup phases=${pickupStarts.join(',')}`)
  semanticAssert(highest < (states[0]?.pos[1] ?? 0) - 16, scenario.name, `highest y=${highest}`)
}
