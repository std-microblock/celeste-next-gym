import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { near, semanticAssert } from '../../verify.js'
import { ENTITY_4_26_JELLYVATOR_PART } from '../glider-parts.js'

export const mapParts = [ENTITY_4_26_JELLYVATOR_PART] as const
export const scenario = defineScenario({
  target: PLAYGROUND_TARGET, status: 'candidate', tags: ['feature:glider'], techniqueIds: ['4.26'], mapParts,
  name: 'entity-4.26-jellyvator', initial: { pos: [60, 496], on_ground: true },
  inputs: Array.from({ length: 72 }, (_, frame) => input({
    move_y: frame === 23 ? 1 : frame >= 42 ? -1 : 0,
    dash_pressed: frame === 42, grab_held: frame <= 22 || frame >= 45,
  })),
  verify: verifyJellyvator,
})

function verifyJellyvator(states: readonly E2EState[]): void {
  const pickup = states.findIndex((state, frame) => frame > 42 && state.state === 8 && state.holding_glider)
  const restored = states.find((state, frame) => frame > pickup && state.state === 0)
  semanticAssert(pickup > 42, scenario.name, `pickup=${pickup}`)
  semanticAssert(Boolean(restored) && near(restored?.speed[1], -240), scenario.name, `restored=${JSON.stringify(restored)}`)
}
