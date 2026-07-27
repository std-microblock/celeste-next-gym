import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_SWIM } from '../common-parts.js'

export const mapParts = [PLAYGROUND_SWIM] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ["feature:swim"],
  mapParts,
  name: 'playground-swim-up',
    initial: { pos: [504, 456], speed: [0, 0], state: 'Swim' },
    inputs: Array.from({ length: 20 }, () => input({ move_y: -1 })),
})


