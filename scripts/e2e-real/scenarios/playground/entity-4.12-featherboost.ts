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
  name: 'entity-4.12-featherboost',
    initial: { pos: [120, 200], speed: [0, 0] },
    inputs: Array.from({ length: 45 }, (_, frame) => input(
      frame >= 27 ? { move_x: 1, move_y: -1 } : {},
    )),
})
