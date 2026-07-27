import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_BUMPER } from '../common-parts.js'

export const mapParts = [PLAYGROUND_BUMPER] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ["feature:bumper"],
  techniqueIds: [],
  mapParts,
  name: 'playground-bumper-right-hold',
    initial: { pos: [611, 206], speed: [0, 0] },
    inputs: Array.from({ length: 80 }, () => input({ move_x: 1 })),
})
