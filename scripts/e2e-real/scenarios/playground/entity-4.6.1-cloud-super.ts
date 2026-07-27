import { inputFrames } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { TECH_ENTITY_4_6_1_CLOUD_HYPER_SUPER } from '../common-parts.js'

export const mapParts = [TECH_ENTITY_4_6_1_CLOUD_HYPER_SUPER] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:cloud'],
  techniqueIds: ['4.6.1'],
  mapParts,
  name: 'entity-4.6.1-cloud-super',
  initial: { pos: [616, 440], speed: [0, 0] },
  inputs: inputFrames(70, (frame) => ({
    move_x: frame >= 24 ? 1 : 0,
    move_y: 0,
    jump_pressed: frame === 28,
    jump_held: frame === 28,
    dash_pressed: frame === 24,
    crouch_dash_pressed: false,
    grab_held: false,
  })),
  verify(states) {
    const launch = states.find((state) => state.state === 0 && Math.abs(state.speed[0] - 260) <= 0.01 && state.speed[1] < -105)
    if (states.some((state) => state.dead) || !launch) {
      throw new Error(`entity-4.6.1-cloud-super: lifted super was ${JSON.stringify(launch)}`)
    }
  },
})
