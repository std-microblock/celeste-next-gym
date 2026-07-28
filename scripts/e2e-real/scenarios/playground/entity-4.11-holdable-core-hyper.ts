import { input, inputFrames } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { near, semanticAssert } from '../../verify.js'
import { TECH_ENTITY_4_11_HOLDABLE_CORE_HYPER } from '../core-heart-squish-parts.js'

export const mapParts = [TECH_ENTITY_4_11_HOLDABLE_CORE_HYPER] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:bounce-block', 'feature:theo-crystal'],
  techniqueIds: ['4.11'],
  mapParts,
  name: 'entity-4.11-holdable-core-hyper',
  initial: { pos: [384, 360], speed: [0, 0], on_ground: true },
  inputs: inputFrames(90, (frame) => input({
    move_x: frame < 42 ? 1 : -1,
    jump_pressed: frame === 36,
    jump_held: frame === 36,
    crouch_dash_pressed: frame === 32,
    grab_held: frame <= 30 || frame >= 42,
  })),
  verify(states) {
    const pickup = states.findIndex((state) => state.state === 8 && state.holding_theo)
    const released = states.findIndex((state, frame) => frame > pickup && !state.holding_theo)
    const hyper = states.findIndex((state, frame) => frame > released
      && state.state === 0 && near(state.speed[0], 325) && state.speed[1] < -52.5)
    const regrab = states.findIndex((state, frame) => frame > hyper && state.state === 8 && state.holding_theo)
    semanticAssert(pickup >= 0 && released > pickup, scenario.name, `pickup=${pickup}, release=${released}`)
    semanticAssert(states.slice(released, released + 6).every((state) => !state.holding_theo), scenario.name,
      'Holdable.CannotHold did not block the immediate regrab')
    semanticAssert(hyper > released, scenario.name, `core hyper=${hyper}`)
    semanticAssert(regrab > hyper, scenario.name, `air regrab=${regrab}`)
  },
})
