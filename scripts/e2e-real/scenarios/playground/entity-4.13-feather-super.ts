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
  name: 'entity-4.13-feather-super',
    initial: { pos: [900, 496], speed: [0, 0] },
    inputs: Array.from({ length: 50 }, (_, frame) => input({
      move_x: 1,
      jump_pressed: frame === 28,
      jump_held: frame >= 28 && frame < 40,
    })),
})
