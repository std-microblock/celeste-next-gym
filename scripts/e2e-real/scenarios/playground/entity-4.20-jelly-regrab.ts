import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { near, semanticAssert } from '../../verify.js'
import { ENTITY_4_20_JELLY_REGRAB_PART } from '../glider-parts.js'

export const mapParts = [ENTITY_4_20_JELLY_REGRAB_PART] as const
export const scenario = defineScenario({
  target: PLAYGROUND_TARGET, status: 'active', tags: ['feature:glider'], techniqueIds: ['4.20'], mapParts,
  name: 'entity-4.20-jelly-regrab', initial: { pos: [60, 496], on_ground: true },
  recording: { primaryFor: ['4.20'], startFrame: 0, endFrame: 72, posterFrame: 47 },
  inputs: Array.from({ length: 72 }, (_, frame) => input({
    move_x: frame >= 24 && frame < 40 ? -1 : frame >= 40 ? 1 : 0,
    move_y: frame === 23 ? 1 : 0, dash_pressed: frame === 40,
    grab_held: frame <= 22 || frame >= 43,
  })),
  verify: verifyJellyRegrab,
})

function verifyJellyRegrab(states: readonly E2EState[]): void {
  const first = states.findIndex((state) => state.state === 8 && state.holding_glider)
  const released = states.findIndex((state, frame) => frame > first && !state.holding_glider)
  const regrab = states.findIndex((state, frame) => frame > released && state.state === 8 && state.holding_glider)
  semanticAssert(first >= 0 && released > first, scenario.name, `pickup=${first}, release=${released}`)
  semanticAssert(regrab > released && near(states[regrab]?.speed[0], 0), scenario.name, `regrab=${regrab}`)
}
