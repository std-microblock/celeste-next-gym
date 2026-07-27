import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_BOOSTER } from '../common-parts.js'

export const mapParts = [PLAYGROUND_BOOSTER] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ["feature:booster"],
  techniqueIds: [],
  mapParts,
  name: 'playground-red-booster-auto',
    initial: { pos: [824, 440], speed: [0, 0] },
    inputs: Array.from({ length: 30 }, () => input()),
})
