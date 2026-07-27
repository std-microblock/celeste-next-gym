import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_2_TARGET } from '../../targets.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_2_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: [],
  mapParts,
  name: 'entity-4.9-dream-grab',
    initial: { pos: [776, -50], speed: [0, 0] },
    inputs: Array.from({ length: 28 }, (_, frame) => input({
      move_x: frame < 15 ? 1 : -1,
      dash_pressed: frame === 0,
      grab_held: frame >= 15,
    })),
})
