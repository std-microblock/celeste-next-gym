import { inputFrames } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { semanticAssert } from '../../verify.js'
import { TECH_DASHLESS_3_7_11_DISAPPEARING_BLOCK_CORNERBOOST } from '../cassette-spinner-parts.js'

export const mapParts = [TECH_DASHLESS_3_7_11_DISAPPEARING_BLOCK_CORNERBOOST] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:cassette-block'],
  techniqueIds: ['3.7.11'],
  mapParts,
  name: 'dashless-3.7.11-disappearing-block-cornerboost',
  initial: { pos: [120, 480], speed: [0, 0] },
  inputs: inputFrames(110, (frame) => ({
    move_x: frame >= 52 ? 1 : 0,
    move_y: 0,
    jump_pressed: frame === 52,
    jump_held: frame >= 52 && frame < 64,
    dash_pressed: false,
    crouch_dash_pressed: false,
    grab_held: false,
  })),
  verify(states) {
    const hit = states.findIndex((state) => state.speed[0] === 0 && state.pos[0] >= 123)
    const refunded = states.findIndex((state, frame) => frame > hit && state.speed[0] > 70)
    semanticAssert(hit > 0 && refunded > hit && !states.some((state) => state.dead), scenario.name,
      `cassette disappearance did not clear the wall and restore horizontal movement: hit=${hit}, refunded=${refunded}`)
  },
})
