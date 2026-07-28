import { input, inputFrames } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { semanticAssert } from '../../verify.js'
import { TECH_ENTITY_4_15_JUMPTHROUGH_CLIP } from '../core-heart-squish-parts.js'

export const mapParts = [TECH_ENTITY_4_15_JUMPTHROUGH_CLIP] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: ['feature:zip-mover', 'feature:squish'],
  techniqueIds: ['4.15'],
  mapParts,
  name: 'entity-4.15-jumpthrough-clip',
  initial: { pos: [652, 304], speed: [0, 0], on_ground: true },
  inputs: inputFrames(100, (frame) => input({
    move_x: frame < 3 ? 1 : frame < 20 ? -1 : 0,
  })),
  verify(states) {
    const landed = states.findIndex((state, frame) => frame > 10 && state.on_ground && state.pos[1] <= 416)
    const clipped = states.findIndex((state, frame) => frame > landed && state.pos[1] > 416 && !state.dead)
    semanticAssert(landed >= 0, scenario.name, 'player never reached the JumpThru before the pusher')
    semanticAssert(clipped > landed, scenario.name, `clip frame=${clipped}, landed=${landed}`)
    semanticAssert(states.slice(0, clipped + 1).every((state) => !state.dead), scenario.name,
      'squish handling killed the player instead of using TargetPosition/wiggle')
  },
})
