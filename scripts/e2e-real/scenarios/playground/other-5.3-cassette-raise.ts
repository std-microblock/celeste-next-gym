import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { semanticAssert } from '../../verify.js'
import { TECH_OTHER_5_3_CASSETTE_RAISE } from '../cassette-spinner-parts.js'

export const mapParts = [TECH_OTHER_5_3_CASSETTE_RAISE] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:cassette-block'],
  techniqueIds: ['5.3'],
  mapParts,
  name: 'other-5.3-cassette-raise',
  initial: { pos: [96, 496], speed: [0, 0], on_ground: true },
  inputs: Array.from({ length: 100 }, () => input()),
  verify(states) {
    const firstPixel = states.findIndex((state) => state.pos[1] < 496 && state.pos[1] > 493)
    const secondPixel = states.findIndex((state) => state.pos[1] <= 493.01)
    semanticAssert(firstPixel > 0 && secondPixel > firstPixel, scenario.name,
      `two-stage cassette raise was not observed: first=${firstPixel}, second=${secondPixel}`)
  },
})
