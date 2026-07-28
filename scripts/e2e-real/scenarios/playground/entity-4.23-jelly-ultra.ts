import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { near, semanticAssert } from '../../verify.js'
import { ENTITY_4_23_JELLY_ULTRA_PART } from '../glider-parts.js'

export const mapParts = [ENTITY_4_23_JELLY_ULTRA_PART] as const
export const scenario = defineScenario({
  target: PLAYGROUND_TARGET, status: 'candidate', tags: ['feature:glider'], techniqueIds: ['4.23'], mapParts,
  name: 'entity-4.23-jelly-ultra', initial: { pos: [32, 160], speed: [300, 0], on_ground: true },
  inputs: Array.from({ length: 24 }, (_, frame) => input({ move_x: 1, move_y: frame < 10 ? 1 : 0, dash_pressed: frame === 0, grab_held: frame >= 5 })),
  verify: verifyJellyUltra,
})

function verifyJellyUltra(states: readonly E2EState[]): void {
  const pickup = states.findIndex((state) => state.state === 8 && state.holding_glider)
  const restored = states.find((state, frame) => frame > pickup && state.state === 0)
  semanticAssert(pickup > 0 && near(states[pickup - 1]?.speed[0], 360), scenario.name, `pickup=${pickup}`)
  semanticAssert(Boolean(restored) && near(restored?.speed[0], 360), scenario.name, `restored=${JSON.stringify(restored)}`)
}
