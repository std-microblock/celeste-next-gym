import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { field, semanticAssert } from '../../verify.js'
import { TECH_OTHER_5_1_1_BINO_CLIP } from '../lookout-parts.js'

export const mapParts = [TECH_OTHER_5_1_1_BINO_CLIP] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:lookout', 'feature:spinner'],
  techniqueIds: ['5.1.1'],
  mapParts,
  name: 'other-5.1.1-bino-clip',
  initial: { pos: [512, 496], speed: [0, 0], on_ground: true },
  inputs: Array.from({ length: 230 }, (_, frame) => input({
    talk_pressed: frame === 0,
    // Move the camera toward x=0. The spinner initially lies in view, then
    // becomes more than a viewport-width to the right and must be hidden by
    // its source 0.25-second visibility interval.
    move_x: frame >= 55 && frame < 205 ? -1 : 0,
    jump_pressed: frame === 205,
    jump_held: frame === 205,
  })),
  verify(states) {
    const visible = states.map((state) => field<boolean>(state, 'crystalSpinnerVisible'))
    const collidable = states.map((state) => field<boolean>(state, 'crystalSpinnerCollidable'))
    const cameras = states.map((state) => field<readonly number[]>(state, 'levelCamera'))
    semanticAssert(visible.some(Boolean), scenario.name, 'spinner never entered its real camera view')
    const firstOffscreen = cameras.findIndex((camera, frame) => frame > 100 && (camera?.[0] ?? 0) < 300)
    semanticAssert(firstOffscreen >= 0, scenario.name,
      'Lookout camera never moved the spinner outside its 320px view')
    const intervalEnd = Math.min(firstOffscreen + 16, states.length)
    semanticAssert(visible.slice(firstOffscreen, intervalEnd).some((value) => value === false), scenario.name,
      'spinner stayed visible for more than one 0.25-second off-screen interval')
    semanticAssert(collidable.slice(firstOffscreen, intervalEnd).some((value) => value === false), scenario.name,
      'spinner stayed Collidable after the off-screen visibility interval')
  },
})
