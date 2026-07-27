import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { TECH_ENTITY_4_3_BUMPER_CLIP } from '../common-parts.js'

export const mapParts = [TECH_ENTITY_4_3_BUMPER_CLIP] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ["feature:bumper"],
  techniqueIds: ['4.3'],
  mapParts,
  name: 'entity-4.3-bumper-clip',
    initial: { pos: [589, 206], speed: [0, 0] },
    inputs: Array.from({ length: 50 }, (_, frame) => input({
      move_x: 1,
      dash_pressed: frame === 20,
    })),
    verify(states) {
      const clipped = states.find((state) => state.pos[0] > 600 && state.state !== 7)
      if (states.some((state) => state.dead) || !states.some((state) => state.state === 2) || !clipped) {
        throw new Error(`entity-4.3-bumper-clip: did not dash through the used bumper: ${JSON.stringify(states.map((state) => ({ frame: state._frame, pos: state.pos, speed: state.speed, state: state.state })))}`)
      }
    },
})
