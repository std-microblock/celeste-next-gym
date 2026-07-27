import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_ICE_BALL } from '../common-parts.js'

export const mapParts = [PLAYGROUND_ICE_BALL] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ["feature:booster"],
  mapParts,
  name: 'entity-4.5-iceball-jump',
    initial: { pos: [317, 155], speed: [0, 0] },
    inputs: Array.from({ length: 24 }, (_, frame) => input({
      move_x: 1,
      move_y: 1,
      jump_held: true,
      dash_pressed: frame === 0,
    })),
})


