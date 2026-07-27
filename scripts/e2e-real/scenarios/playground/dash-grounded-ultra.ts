import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_BASE } from '../common-parts.js'
import { verifyGroundedUltra } from '../shared/ultra.js'

export const mapParts = [PLAYGROUND_BASE] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  mapParts,
  name: 'dash-grounded-ultra',
    initial: { pos: [820, 496], speed: [300, 0] },
    inputs: Array.from({ length: 12 }, (_, frame) => input({
      move_x: 1,
      move_y: 1,
      dash_pressed: frame === 0,
    })),
    verify: verifyGroundedUltra,
})


