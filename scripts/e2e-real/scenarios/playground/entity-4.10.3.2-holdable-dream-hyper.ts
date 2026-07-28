import { inputFrames } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { TECH_ENTITY_4_10_3_2_HOLDABLE_DREAM_HYPER } from '../common-parts.js'

export const mapParts = [TECH_ENTITY_4_10_3_2_HOLDABLE_DREAM_HYPER] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ['feature:dream-block', 'feature:theo'],
  techniqueIds: ['4.10.3.2'],
  mapParts,
  name: 'entity-4.10.3.2-holdable-dream-hyper',
  recording: { primaryFor: ['4.10.3.2'], startFrame: 0, endFrame: 240 },
  initial: { pos: [208, 496], speed: [0, 0], can_dream_dash: true },
  inputs: inputFrames(240, (frame) => ({
    move_x: frame >= 43 && frame < 54 ? -1 : frame >= 85 ? -1 : 1,
    move_y: 0,
    jump_pressed: frame === 62,
    jump_held: frame === 62,
    dash_pressed: frame === 0,
    crouch_dash_pressed: frame === 54,
    grab_held: frame < 52 || frame >= 65,
  })),
  verify(states) {
    const exitGrab = states.findIndex((state, frame) => frame > 0 && state.state === 1 && states[frame - 1]?.state === 9 && state.holding_theo)
    const released = states.findIndex((state, frame) => frame > exitGrab && !state.holding_theo)
    const blockedRegrab = released >= 0 && states.slice(released + 1, released + 6).every((state) => !state.holding_theo)
    const hyper = states.findIndex((state, frame) => frame > released && state.state === 0 && Math.abs(state.speed[0] - 325) <= 0.01)
    const regrab = states.find((state, frame) => frame > hyper && state.state === 8 && state.holding_theo)
    if (states.some((state) => state.dead) || exitGrab < 0 || released < 0 || !blockedRegrab || hyper < 0 || !regrab) {
      throw new Error(`entity-4.10.3.2-holdable-dream-hyper: grab=${exitGrab}, release=${released}, blocked=${blockedRegrab}, hyper=${hyper}, regrab=${!!regrab}`)
    }
  },
})
