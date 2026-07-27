import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_STAR_FLY, PLAYGROUND_ICE_BALL } from '../common-parts.js'

export const mapParts = [PLAYGROUND_STAR_FLY, PLAYGROUND_ICE_BALL] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ["feature:booster"],
  techniqueIds: [],
  mapParts,
  name: 'entity-4.15.2-feather-hitbox-preservation',
    initial: { pos: [320, 120], speed: [0, 0] },
    inputs: Array.from({ length: 60 }, () => input({ move_y: 1 })),
})
