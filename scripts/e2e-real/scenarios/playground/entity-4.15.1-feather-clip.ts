import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { pickCore } from '../../verify.js'
import { TECH_ENTITY_4_15_1_FEATHER_CLIP } from '../common-parts.js'

export const mapParts = [TECH_ENTITY_4_15_1_FEATHER_CLIP] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ['feature:star-fly'],
  techniqueIds: ['4.15.1'],
  mapParts,
  name: 'entity-4.15.1-feather-clip',
  recording: { primaryFor: ['4.15.1'], startFrame: 0, endFrame: 180 },
  initial: { pos: [160, 40], speed: [0, 0] },
  inputs: Array.from({ length: 180 }, () => input({ move_y: 1 })),
  verify(states) {
    const exitIndex = states.findIndex((state, index) => index > 0
      && states[index - 1]?.state === 19 && state.state === 0)
    const exit = states[exitIndex]
    if (!exit || exit.pos[1] < 402 || exit.on_ground || exit.dead) {
      throw new Error(`entity-4.15.1-feather-clip: StarFly did not expire below the isolated 16px jumpthrough: ${JSON.stringify({
        exit: pickCore(exit),
        neighbors: states.slice(Math.max(0, exitIndex - 2), exitIndex + 3).map(pickCore),
      })}`)
    }
  },
})
