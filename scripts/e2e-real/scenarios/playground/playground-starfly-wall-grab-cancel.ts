import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_STAR_FLY } from '../common-parts.js'

export const mapParts = [PLAYGROUND_STAR_FLY] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ["feature:star-fly"],
  mapParts,
  name: 'playground-starfly-wall-grab-cancel',
    initial: { pos: [120, 200], speed: [0, 0] },
    inputs: Array.from({ length: 150 }, (_, frame) => input({
      move_x: 1,
      grab_held: frame >= 120,
    })),
})


