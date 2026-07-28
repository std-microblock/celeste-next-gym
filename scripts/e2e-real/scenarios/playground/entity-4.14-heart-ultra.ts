import { input, inputFrames } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { field, near, semanticAssert } from '../../verify.js'
import { TECH_ENTITY_4_14_HEART_ULTRA } from '../core-heart-squish-parts.js'

export const mapParts = [TECH_ENTITY_4_14_HEART_ULTRA] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:heart-gem'],
  techniqueIds: ['4.14'],
  mapParts,
  name: 'entity-4.14-heart-ultra',
  initial: { pos: [484, 496], speed: [300, 0], on_ground: true },
  inputs: inputFrames(28, (frame) => input({
    move_x: 1,
    move_y: 1,
    crouch_dash_pressed: frame === 0,
  })),
  verify(states) {
    const collected = states.findIndex((state) => field(state, 'heartGemCollected') === true)
    const ultra = states.findIndex((state) => state.state === 2 && state.ducking && near(state.speed[0], 360))
    const heartFreeze = states.findIndex((state, frame) => frame > collected && Number(state.freeze_timer) >= 0.19)
    const halfTime = states.findIndex((state, frame) => frame > heartFreeze && near(Number(field(state, 'engineTimeRate')), 0.5))
    semanticAssert(collected >= 0, scenario.name, 'HeartGem never entered its collected state')
    semanticAssert(ultra >= 0 && Math.abs(ultra - collected) <= 1, scenario.name,
      `down-diagonal ultra=${ultra}, collect=${collected}`)
    semanticAssert(heartFreeze > collected, scenario.name, `heart freeze=${heartFreeze}`)
    semanticAssert(halfTime > heartFreeze, scenario.name, `half time=${halfTime}`)
  },
})
