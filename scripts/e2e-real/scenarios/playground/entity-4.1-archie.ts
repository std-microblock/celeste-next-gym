import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_BOOSTER } from '../common-parts.js'

export const mapParts = [PLAYGROUND_BOOSTER] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ["feature:booster"],
  techniqueIds: ['4.1'],
  recording: { primaryFor: ['4.1'], startFrame: 0, endFrame: 36 },
  mapParts,
  name: 'entity-4.1-archie',
    initial: { pos: [680, 330], speed: [0, 0] },
    inputs: Array.from({ length: 36 }, (_, frame) => input({
      move_x: 1,
      crouch_dash_pressed: frame === 0,
    })),
})
