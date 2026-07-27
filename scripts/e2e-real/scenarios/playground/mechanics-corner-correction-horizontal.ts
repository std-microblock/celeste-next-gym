import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_DASHLESS } from '../common-parts.js'

export const mapParts = [PLAYGROUND_DASHLESS] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  mapParts,
  name: 'mechanics-corner-correction-horizontal',
    initial: { pos: [392, 82], speed: [0, 0] },
    inputs: Array.from({ length: 12 }, (_, frame) => input({
      move_x: 1,
      dash_pressed: frame === 0,
    })),
})


