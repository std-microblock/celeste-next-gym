import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_STAR_FLY } from '../common-parts.js'

export const mapParts = [PLAYGROUND_STAR_FLY] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ["feature:star-fly"],
  techniqueIds: [],
  mapParts,
  name: 'entity-4.15.1-feather-clip',
    initial: { pos: [160, 40], speed: [0, 0] },
    inputs: Array.from({ length: 180 }, () => input({ move_y: 1 })),
})
