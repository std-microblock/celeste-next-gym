import { inputFrames } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { semanticAssert } from '../../verify.js'
import { cassetteBlock } from '../cassette-observation.js'
import { TECH_DASHLESS_3_7_11_DISAPPEARING_BLOCK_CORNERBOOST } from '../cassette-spinner-parts.js'

export const mapParts = [TECH_DASHLESS_3_7_11_DISAPPEARING_BLOCK_CORNERBOOST] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ['feature:cassette-block'],
  techniqueIds: ['3.7.11'],
  recording: { primaryFor: ['3.7.11'], startFrame: 0, endFrame: 60 },
  mapParts,
  name: 'dashless-3.7.11-disappearing-block-cornerboost',
  initial: { pos: [120, 496], speed: [0, 0], on_ground: true },
  inputs: inputFrames(60, (frame) => ({
    // With a fresh custom manager, tempo=3 reaches beat 8 after input 28.
    // Starting at 22 makes the grounded 90-speed run hit index 1 before the
    // beat-8 manager update writes Activated=false. Its next entity phase
    // clears collision, and the following Player.Update restores retained
    // speed inside the 0.06s window.
    move_x: frame >= 22 ? 1 : 0,
    move_y: 0,
    jump_pressed: false,
    jump_held: false,
    dash_pressed: false,
    crouch_dash_pressed: false,
    grab_held: false,
  })),
  verify(states) {
    const hit = states.findIndex((state) => state.speed[0] === 0 && state.pos[0] >= 123)
    // CassetteBlock.Update runs after the Player collision in the same raw
    // frame, so the post-frame snapshot can already show its cleared wall.
    // The following Player update must then restore the retained wall speed.
    const disappeared = states.findIndex((state, frame) => frame >= hit && cassetteBlock(state, 1)?.collidable === false)
    const refunded = states.findIndex((state, frame) => frame > hit && state.speed[0] > 70)
    semanticAssert(hit > 0 && disappeared >= hit && refunded > hit && !states.some((state) => state.dead), scenario.name,
      `cassette wall did not impact, disappear, then refund speed: hit=${hit}, disappeared=${disappeared}, refunded=${refunded}`)
  },
})
