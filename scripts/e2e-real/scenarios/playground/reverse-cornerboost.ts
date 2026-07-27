import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { near, semanticAssert } from '../../verify.js'
import { REVERSE_CORNERBOOST_PART } from '../dashless-cornerboost-parts.js'

export const mapParts = [REVERSE_CORNERBOOST_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: [],
  techniqueIds: ['3.7.8'],
  mapParts,
  name: 'reverse-cornerboost',
  initial: { pos: [132, 242], speed: [160, -30], facing: 'Left' },
  inputs: Array.from({ length: 20 }, (_, frame) => input({
    move_x: frame === 0 ? -1 : 1,
    jump_pressed: frame === 0,
    jump_held: frame < 12,
    grab_held: frame === 0,
  })),
  verify: verifyReverseCornerboost,
})

function verifyReverseCornerboost(states: readonly E2EState[]): void {
  const jumped = states[1]
  semanticAssert(jumped?.state === 0 && jumped.facing === false, 'reverse-cornerboost', 'backward climb jump did not remain in Normal facing left')
  semanticAssert(near(jumped?.stamina, 82.5) && near(jumped?.speed[0], 109.166_664), 'reverse-cornerboost', `backward boost produced ${JSON.stringify(jumped?.speed)} at stamina ${jumped?.stamina}`)
  semanticAssert(states.every((state) => !state.dead), 'reverse-cornerboost', 'reverse cornerboost died in its isolated booth')
}
