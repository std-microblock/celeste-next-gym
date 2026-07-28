import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { near, pickCore } from '../../verify.js'
import { TECH_ENTITY_4_12_FEATHERBOOST } from '../common-parts.js'

export const mapParts = [TECH_ENTITY_4_12_FEATHERBOOST] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ['feature:star-fly'],
  techniqueIds: ['4.12'],
  mapParts,
  name: 'entity-4.12-featherboost',
  recording: { primaryFor: ['4.12'], startFrame: 0, endFrame: 45 },
  initial: { pos: [120, 200], speed: [0, 0] },
  inputs: Array.from({ length: 45 }, (_, frame) => input(
    frame >= 27 ? { move_x: 1, move_y: -1 } : {},
  )),
  verify(states) {
    const launched = states.find((state) => state.state === 19
      && near(state.speed[0], 176.776_69, 0.01)
      && near(state.speed[1], -176.776_69, 0.01))
    if (!launched) {
      throw new Error(`entity-4.12-featherboost: missing first-live-frame diagonal 250 launch: ${JSON.stringify(
        states.slice(25, 31).map(pickCore),
      )}`)
    }
  },
})
