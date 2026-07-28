import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { near, semanticAssert } from '../../verify.js'
import { ENTITY_4_22_3_JELLY_NEUTRAL_PART } from '../glider-parts.js'

export const mapParts = [ENTITY_4_22_3_JELLY_NEUTRAL_PART] as const
export const scenario = defineScenario({
  target: PLAYGROUND_TARGET, status: 'candidate', tags: ['feature:glider'], techniqueIds: ['4.22.3'], mapParts,
  name: 'entity-4.22.3-jelly-neutral-jump', initial: { pos: [140, 420], facing: 'Right' },
  inputs: Array.from({ length: 100 }, (_, frame) => input({
    move_x: frame >= 30 ? 1 : 0,
    move_y: frame === 23 ? 1 : 0, jump_pressed: frame === 24, jump_held: frame === 24,
    grab_held: frame <= 22 || frame >= 25,
  })),
  verify: verifyJellyNeutral,
})

function verifyJellyNeutral(states: readonly E2EState[]): void {
  const released = states.findIndex((state, frame) => frame > 0 && !state.holding_glider && states[frame - 1]?.holding_glider)
  const neutral = states.findIndex((state, frame) => frame > released && near(state.speed[0], -130) && near(state.speed[1], -105))
  const regrab = states.findIndex((state, frame) => frame > neutral && state.state === 8 && state.holding_glider)
  semanticAssert(released >= 0 && neutral > released, scenario.name, `release=${released}, neutral=${neutral}`)
  semanticAssert(regrab > neutral, scenario.name, `regrab=${regrab}`)
}
