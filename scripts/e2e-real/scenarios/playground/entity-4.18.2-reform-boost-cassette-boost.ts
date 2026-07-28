import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { semanticAssert } from '../../verify.js'
import { TECH_ENTITY_4_18_2_REFORM_BOOST } from '../cassette-spinner-parts.js'

export const mapParts = [TECH_ENTITY_4_18_2_REFORM_BOOST] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ['feature:cassette-block'],
  techniqueIds: ['4.18.2'],
  mapParts,
  name: 'entity-4.18.2-reform-boost-cassette-boost',
  recording: { primaryFor: ['4.18.2'], startFrame: 0, endFrame: 100, posterFrame: 82 },
  initial: { pos: [96, 496], speed: [0, 0], on_ground: true },
  inputs: Array.from({ length: 100 }, () => input()),
  verify(states) {
    const lifted = states.findIndex((state) => state.pos[1] <= 493.01)
    semanticAssert(lifted > 0, scenario.name, `cassette reform never displaced the player upward: ${lifted}`)
    semanticAssert(!states.some((state) => state.dead), scenario.name, 'cassette reform killed the player')
  },
})
