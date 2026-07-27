import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { near, pickCore } from '../../verify.js'
import { TECH_ENTITY_4_13_FEATHER_SUPER } from '../common-parts.js'

export const mapParts = [TECH_ENTITY_4_13_FEATHER_SUPER] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ['feature:star-fly'],
  techniqueIds: ['4.13'],
  mapParts,
  name: 'entity-4.13-feather-super',
  initial: { pos: [900, 496], speed: [0, 0] },
  inputs: Array.from({ length: 50 }, (_, frame) => input({
    move_x: 1,
    jump_pressed: frame === 28,
    jump_held: frame >= 28 && frame < 40,
  })),
  verify(states) {
    const jumped = states.find((state) => state.state === 0
      && near(state.speed[0], 273.333_34, 0.01)
      && near(state.speed[1], -105, 0.01))
    if (!jumped || jumped.on_ground || jumped.dead) {
      throw new Error(`entity-4.13-feather-super: missing grounded horizontal feather exit jump: ${JSON.stringify(
        states.slice(26, 33).map(pickCore),
      )}`)
    }
  },
})
