import { inputFrames } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { TECH_ENTITY_4_6_2_CLOUD_HYPER_BUNNYHOP } from '../common-parts.js'

export const mapParts = [TECH_ENTITY_4_6_2_CLOUD_HYPER_BUNNYHOP] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:cloud'],
  techniqueIds: ['4.6.2'],
  mapParts,
  name: 'entity-4.6.2-cloud-hyper-bunnyhop',
  initial: { pos: [520, 440], speed: [0, 0] },
  inputs: inputFrames(110, (frame) => ({
    move_x: frame >= 24 ? 1 : 0,
    move_y: 0,
    jump_pressed: frame === 28 || frame === 62,
    jump_held: frame === 28 || frame === 62,
    dash_pressed: false,
    crouch_dash_pressed: frame === 24,
    grab_held: false,
  })),
  verify(states) {
    const hyper = states.findIndex((state) => state.state === 0 && Math.abs(state.speed[0] - 325) <= 0.01)
    const bunnyhop = states.find((state, frame) => frame > hyper && state.state === 0 && state.speed[0] > 250 && state.speed[1] < -105)
    if (states.some((state) => state.dead) || hyper < 0 || !bunnyhop) {
      throw new Error(`entity-4.6.2-cloud-hyper-bunnyhop: hyper=${hyper}, bunnyhop=${JSON.stringify(bunnyhop)}`)
    }
  },
})
