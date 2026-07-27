import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_WIND } from '../common-parts.js'

export const mapParts = [PLAYGROUND_WIND] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ["feature:wind"],
  techniqueIds: ['1.5'],
  recording: { primaryFor: ['1.5'], startFrame: 0, endFrame: 40 },
  mapParts,
  name: 'mechanics-dash-attack-late-shield',
    initial: { pos: [55, 120], speed: [0, 0] },
    inputs: Array.from({ length: 40 }, (_, frame) => input({
      move_x: 1,
      dash_pressed: frame === 0,
    })),
})
