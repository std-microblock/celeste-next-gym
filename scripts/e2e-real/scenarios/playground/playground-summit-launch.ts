import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_LAUNCH } from '../common-parts.js'

export const mapParts = [PLAYGROUND_LAUNCH] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ["feature:launch"],
  mapParts,
  name: 'playground-summit-launch',
    initial: { pos: [500, 400], speed: [0, 0], state: 'SummitLaunch' },
    inputs: Array.from({ length: 60 }, () => input()),
})


