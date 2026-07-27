import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_DASHLESS } from '../common-parts.js'

export const mapParts = [PLAYGROUND_DASHLESS] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: [],
  mapParts,
  name: 'eight-jump',
    initial: { pos: [458, 120], speed: [0, 0], on_ground: true },
    inputs: Array.from({ length: 120 }, (_, frame) => input({
      move_x: 1,
      jump_pressed: frame === 5 || frame === 11 || frame === 12 || frame === 13,
      jump_held: frame <= 26,
      grab_held: frame === 11 || frame === 12 || frame === 13,
    })),
})
