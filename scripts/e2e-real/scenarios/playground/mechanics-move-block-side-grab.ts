import { input, inputFrames } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { pickCore, semanticAssert } from '../../verify.js'
import { MOVE_BLOCK_SIDE_GRAB_PART } from '../side-grab-parts.js'

export const mapParts = [MOVE_BLOCK_SIDE_GRAB_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ['feature:move-block'],
  techniqueIds: [],
  mapParts,
  name: 'mechanics-move-block-side-grab',
  initial: { pos: [636, 408], speed: [0, 0], facing: 'Left', stamina: 110 },
  inputs: inputFrames(40, () => input({ grab_held: true })),
  verify(states) {
    const grabbed = states.findIndex((state) => state.state === 'Climb' || state.state === 1)
    const carried = states.findIndex((state, frame) => frame > grabbed
      && (state.state === 'Climb' || state.state === 1)
      && state.pos[0] < 635.99)
    semanticAssert(grabbed >= 0, scenario.name,
      `stationary left-facing player did not grab the MoveBlock: ${JSON.stringify(states.slice(0, 4).map(pickCore))}`)
    semanticAssert(carried > grabbed, scenario.name,
      `side grab did not activate and ride the left-moving MoveBlock: ${JSON.stringify(states.map(pickCore))}`)
    semanticAssert(!states.some((state) => state.dead), scenario.name, 'player died while riding the MoveBlock side')
  },
})
