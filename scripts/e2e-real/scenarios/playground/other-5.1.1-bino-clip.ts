import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { field, semanticAssert } from '../../verify.js'
import { TECH_OTHER_5_1_1_BINO_CLIP } from '../lookout-parts.js'

export const mapParts = [TECH_OTHER_5_1_1_BINO_CLIP] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ['feature:lookout', 'feature:spinner'],
  techniqueIds: ['5.1.1'],
  mapParts,
  recording: { primaryFor: ['5.1.1'], startFrame: 0, endFrame: 230, posterFrame: 90 },
  name: 'other-5.1.1-bino-clip',
  initial: { pos: [512, 496], speed: [0, 0], on_ground: true },
  inputs: Array.from({ length: 230 }, (_, frame) => input({
    talk_pressed: frame === 0,
    // Aim away from the spinner: its center is x=636, so Camera.X below
    // 300 places it past CrystalStaticSpinner.InView's right edge.
    move_x: frame >= 55 && frame < 205 ? -1 : 0,
    jump_pressed: frame === 205,
    jump_held: frame === 205,
  })),
  verify(states) {
    const cameras = states.map((state) => field<readonly number[]>(state, 'levelCamera'))
    const visible = states.map((state) => field<boolean>(state, 'crystalSpinnerVisible'))
    const collidable = states.map((state) => field<boolean>(state, 'crystalSpinnerCollidable'))
    semanticAssert(visible.some(Boolean), scenario.name, 'spinner never entered its real camera view')
    const offscreenFrame = cameras.findIndex((camera) => (camera?.[0] ?? Infinity) < 300)
    semanticAssert(offscreenFrame >= 0, scenario.name,
      'Lookout camera never moved left past the spinner InView boundary')
    const hiddenWithinInterval = visible.slice(offscreenFrame, offscreenFrame + 16)
      .some((value, index) => value === false && collidable[offscreenFrame + index] === false)
    semanticAssert(hiddenWithinInterval, scenario.name,
      'spinner did not become invisible and non-collidable within the 0.25-second off-screen interval')
  },
})
