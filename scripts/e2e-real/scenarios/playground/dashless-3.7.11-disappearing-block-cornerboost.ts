import { inputFrames } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { semanticAssert } from '../../verify.js'
import { cassetteBlock } from '../cassette-observation.js'
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
    // Index 1 starts active. Enter its right wall just before the first
    // 8-beat color switch, so it remains a Solid for impact then disappears
    // before wall-speed retention expires.
    move_x: frame >= 75 ? 1 : 0,
    move_y: 0,
    jump_pressed: false,
    jump_held: false,
    dash_pressed: false,
    crouch_dash_pressed: false,
    grab_held: false,
  })),
  verify(states) {
    const hit = states.findIndex((state) => state.speed[0] === 0 && state.pos[0] >= 123)
    const disappeared = states.findIndex((state, frame) => frame > hit && cassetteBlock(state, 1)?.collidable === false)
    const refunded = states.findIndex((state, frame) => frame > disappeared && state.speed[0] > 70)
    semanticAssert(hit > 0 && disappeared > hit && refunded > disappeared && !states.some((state) => state.dead), scenario.name,
      `cassette wall did not impact, disappear, then refund speed: hit=${hit}, disappeared=${disappeared}, refunded=${refunded}`)
  },
})
