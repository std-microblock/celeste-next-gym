import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { semanticAssert } from '../../verify.js'
import { TECH_OTHER_5_11_SPINNER_FREEZE } from '../cassette-spinner-parts.js'

export const mapParts = [TECH_OTHER_5_11_SPINNER_FREEZE] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:spinner', 'long-runtime'],
  techniqueIds: ['5.11'],
  mapParts,
  name: 'other-5.11-spinner-freeze',
  initial: { pos: [220, 406], speed: [0, 0] },
  inputs: Array.from({ length: 20 }, () => input()),
  verify(states) {
    semanticAssert(states.some((state) => state.dead), scenario.name,
      'baseline spinner did not activate; high-TimeActive grouping cannot be compared')
  },
})
