import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { field, near, semanticAssert } from '../../verify.js'
import { CORNERSLIP_PART } from '../dashless-spike-parts.js'

export const mapParts = [CORNERSLIP_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: [],
  techniqueIds: ['3.13'],
  mapParts,
  name: 'cornerslip',
  initial: { pos: [37, 40], speed: [-90, 60], dashes: 0, can_dream_dash: false },
  inputs: Array.from({ length: 6 }, () => input({ move_x: -1 })),
  verify: verifyCornerslip,
})

function verifyCornerslip(states: readonly E2EState[]): void {
  const slipped = states[1]
  semanticAssert(near(slipped?.pos[0], 35) && near(slipped?.pos[1], 41), 'cornerslip', `corner traversal ended at ${JSON.stringify(slipped?.pos)}`)
  semanticAssert(near(slipped?.speed[0], -90) && near(slipped?.speed[1], 60), 'cornerslip', `vertical collision changed speed to ${JSON.stringify(slipped?.speed)}`)
  semanticAssert(slipped?.dashes === 1 && slipped.on_ground === false, 'cornerslip', 'one-pixel ground probe did not refill before the no-collision slip')
  semanticAssert(Number(field(slipped, 'jumpGraceTimer')) > 0.09, 'cornerslip', `coyote timer was ${String(field(slipped, 'jumpGraceTimer'))}`)
}
