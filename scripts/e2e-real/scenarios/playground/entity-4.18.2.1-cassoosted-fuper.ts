import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { near, semanticAssert } from '../../verify.js'
import { TECH_ENTITY_4_18_2_1_CASSOOSTED_FUPER } from '../cassette-spinner-parts.js'

export const mapParts = [TECH_ENTITY_4_18_2_1_CASSOOSTED_FUPER] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:cassette-block', 'feature:star-fly'],
  techniqueIds: ['4.18.2.1'],
  mapParts,
  name: 'entity-4.18.2.1-cassoosted-fuper',
  initial: { pos: [500, 496], speed: [0, 0], on_ground: true },
  inputs: Array.from({ length: 100 }, (_, frame) => input({
    move_x: frame === 80 ? 1 : 0,
    jump_pressed: frame === 80,
    jump_held: frame >= 80 && frame < 92,
  })),
  verify(states) {
    const fuper = states.findIndex((state) => state.state === 0
      && near(state.speed[0], 273.333_34, 0.01)
      && near(state.speed[1], -105, 0.01))
    semanticAssert(fuper > 0, scenario.name, `grounded feather exit was not observed: ${fuper}`)
    semanticAssert(!states.some((state) => state.dead), scenario.name, 'cassoosted fuper killed the player')
  },
})
