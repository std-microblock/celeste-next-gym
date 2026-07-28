import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { near, semanticAssert } from '../../verify.js'
import { ENTITY_4_23_THEO_ULTRA_PART } from '../entity-tail-parts.js'

export const mapParts = [ENTITY_4_23_THEO_ULTRA_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET, status: 'candidate', tags: [], techniqueIds: ['4.23'], mapParts,
  name: 'entity-4.23-theo-ultra',
  initial: { pos: [32, 160], speed: [300, 0], on_ground: true },
  inputs: Array.from({ length: 24 }, (_, frame) => input({
    move_x: 1, move_y: frame < 10 ? 1 : 0, dash_pressed: frame === 0, grab_held: frame >= 5 && frame <= 20,
  })),
  verify(states) {
    const pickup = states.find((state) => state.state === 8 && state.holding_theo)
    const restored = states.find((state, frame) => frame > 0 && states[frame - 1]?.state === 8 && state.state === 0)
    semanticAssert(Boolean(pickup) && near(pickup?.speed[0], 0), scenario.name, 'Theo pickup did not cancel the grounded ultra into Pickup')
    semanticAssert(Boolean(restored) && near(restored?.speed[0], 360), scenario.name, `restored=${JSON.stringify(restored)}`)
  },
})
