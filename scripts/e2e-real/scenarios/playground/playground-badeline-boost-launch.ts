import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_BADELINE } from '../common-parts.js'

export const mapParts = [PLAYGROUND_BADELINE] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ["feature:badeline-boost"],
  techniqueIds: [],
  mapParts,
  name: 'playground-badeline-boost-launch',
    initial: { pos: [320, 400], speed: [0, 0] },
    inputs: Array.from({ length: 120 }, () => input()),
})
