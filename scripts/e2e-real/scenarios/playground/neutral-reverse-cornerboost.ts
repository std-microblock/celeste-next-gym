import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { field, near, semanticAssert } from '../../verify.js'
import { NEUTRAL_REVERSE_CORNERBOOST_PART } from '../dashless-cornerboost-parts.js'

export const mapParts = [NEUTRAL_REVERSE_CORNERBOOST_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: [],
  techniqueIds: ['3.7.9'],
  mapParts,
  name: 'neutral-reverse-cornerboost',
  initial: { pos: [132, 242], speed: [160, -30], facing: 'Left' },
  inputs: Array.from({ length: 20 }, (_, frame) => input({
    move_x: frame === 0 ? 0 : 1,
    jump_pressed: frame === 0,
    jump_held: frame < 12,
    grab_held: frame === 0,
  })),
  verify: verifyNeutralReverseCornerboost,
})

function verifyNeutralReverseCornerboost(states: readonly E2EState[]): void {
  semanticAssert(near(states[1]?.speed[0], 149.166_64) && near(states[1]?.stamina, 82.5), 'neutral-reverse-cornerboost', 'neutral climb jump did not avoid the backward 40-speed boost')
  semanticAssert(Number(field(states[1], 'wallBoostTimer')) > 0.19, 'neutral-reverse-cornerboost', `wallboost timer was ${String(field(states[1], 'wallBoostTimer'))}`)
  semanticAssert(near(states[3]?.speed[0], 125.666_66) && near(states[3]?.stamina, 110), 'neutral-reverse-cornerboost', `wallboost conversion produced ${JSON.stringify(states[3]?.speed)} at stamina ${states[3]?.stamina}`)
}
