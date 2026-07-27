import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_2_TARGET } from '../../targets.js'
import { field, near, pickCore } from '../../verify.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_2_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['4.10.2'],
  recording: { primaryFor: ['4.10.2'], startFrame: 0, endFrame: 38 },
  mapParts,
  name: 'entity-4.10.2-dream-hyper',
    initial: { pos: [776, -50], speed: [0, 0] },
    inputs: Array.from({ length: 38 }, (_, frame) => input({
      move_x: 1,
      jump_pressed: frame === 24,
      jump_held: frame >= 24 && frame < 34,
      dash_pressed: frame === 0,
      crouch_dash_pressed: frame === 17,
    })),
    verify(states) {
      const hyper = states.find((state) => state.state === 0
        && Math.abs(state.speed[0] - 325) <= 0.01
        && Math.abs(state.speed[1] + 52.5) <= 0.01)
      if (!hyper) throw new Error('entity-4.10.2-dream-hyper: did not execute the expected 325/-52.5 dream-exit hyper')
    },
})
