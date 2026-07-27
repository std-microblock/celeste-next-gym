import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_BASE } from '../common-parts.js'
import type { E2EState } from '../../types.js'
import { field, near, pickCore, semanticAssert } from '../../verify.js'

export const mapParts = [PLAYGROUND_BASE] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['2.9'],
  recording: { primaryFor: ['2.9'], startFrame: 0, endFrame: 30 },
  mapParts,
  name: 'dash-demodash-gap',
    initial: { pos: [712, 320], speed: [0, 0] },
    inputs: Array.from({ length: 30 }, (_, frame) => input({
      move_x: 1,
      crouch_dash_pressed: frame === 0,
    })),
    verify: verifyDemodashGap,
})


function verifyDemodashGap(states: readonly E2EState[]): void {
  semanticAssert(states.some((state) => state.ducking && state.pos[0] > 720), 'dash-demodash-gap', 'crouched dash never entered the six-pixel tunnel')
  const last = states.at(-1)
  semanticAssert(last && last.pos[0] > 760 && !last.dead, 'dash-demodash-gap', 'demo did not progress through the low tunnel')
}
