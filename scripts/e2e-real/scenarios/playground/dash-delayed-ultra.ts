import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_BASE } from '../common-parts.js'
import { verifyDelayedUltra } from '../shared/ultra.js'

export const mapParts = [PLAYGROUND_BASE] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: [],
  mapParts,
  name: 'dash-delayed-ultra',
    initial: { pos: [200, 420], speed: [0, 0] },
    inputs: Array.from({ length: 36 }, (_, frame) => input({
      move_x: 1,
      move_y: 1,
      dash_pressed: frame === 0,
    })),
    verify: verifyDelayedUltra,
})
