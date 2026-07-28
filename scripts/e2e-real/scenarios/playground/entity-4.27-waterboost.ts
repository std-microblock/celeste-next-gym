import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { near, semanticAssert } from '../../verify.js'
import { ENTITY_4_27_WATERBOOST_PART } from '../entity-tail-parts.js'

export const mapParts = [ENTITY_4_27_WATERBOOST_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET, status: 'active', tags: ['feature:swim'], techniqueIds: ['4.27'], mapParts,
  name: 'entity-4.27-waterboost',
  recording: { primaryFor: ['4.27'], startFrame: 0, endFrame: 24, posterFrame: 3 },
  initial: { pos: [136, 68], speed: [0, 0], state: 'Swim' },
  inputs: Array.from({ length: 24 }, (_, frame) => input({
    move_x: 1, jump_pressed: frame <= 2,
  })),
  verify(states) {
    semanticAssert(near(states[1]?.speed[0], 50), scenario.name, `state1=${JSON.stringify(states[1])}`)
    semanticAssert(near(states[2]?.speed[0], 100), scenario.name, `state2=${JSON.stringify(states[2])}`)
    semanticAssert(near(states[3]?.speed[0], 135.666_66), scenario.name, `state3=${JSON.stringify(states[3])}`)
    semanticAssert(near(states[3]?.speed[1], -105), scenario.name, `state3=${JSON.stringify(states[3])}`)
  },
})
