import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_BUMPER } from '../common-parts.js'

export const mapParts = [PLAYGROUND_BUMPER] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ["feature:bumper"],
  mapParts,
  name: 'entity-4.3-bumper-clip',
    initial: { pos: [589, 206], speed: [0, 0] },
    inputs: Array.from({ length: 50 }, (_, frame) => input({
      move_x: 1,
      dash_pressed: frame === 4,
    })),
})


