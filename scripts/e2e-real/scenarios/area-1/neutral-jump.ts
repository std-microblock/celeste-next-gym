import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['3.5'],
  recording: { primaryFor: ['3.5'], startFrame: 0, endFrame: 50 },
  mapParts,
  name: 'neutral-jump',
    initial: { pos: [140, 112], speed: [0, 30] },
    inputs: Array.from({ length: 50 }, (_, frame) => input({
      move_x: frame === 0 || frame === 26 ? 0 : 1,
      jump_pressed: frame === 0 || frame === 26,
      jump_held: frame < 10 || (frame >= 26 && frame < 36),
    })),
})
