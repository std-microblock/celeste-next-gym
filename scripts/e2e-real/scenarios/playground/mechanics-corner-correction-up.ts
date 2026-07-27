import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_BASE } from '../common-parts.js'

export const mapParts = [PLAYGROUND_BASE] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  mapParts,
  name: 'mechanics-corner-correction-up',
    initial: { pos: [477, 275], speed: [0, -105] },
    inputs: Array.from({ length: 8 }, () => input()),
})


