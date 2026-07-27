import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_MISC } from '../common-parts.js'

export const mapParts = [PLAYGROUND_MISC] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ["feature:misc-states"],
  mapParts,
  name: 'playground-frozen-state',
    initial: { pos: [600, 300], speed: [60, 30], state: 'Frozen' },
    inputs: Array.from({ length: 20 }, () => input()),
})


