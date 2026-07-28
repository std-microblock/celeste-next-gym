import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { near, pickCore, semanticAssert } from '../../verify.js'
import { PLAYGROUND_DASHLESS } from '../common-parts.js'

export const mapParts = [PLAYGROUND_DASHLESS] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['1.3'],
  mapParts,
  name: 'mechanics-corner-correction-down',
  initial: { pos: [251, 96], speed: [0, 0] },
  inputs: Array.from({ length: 12 }, (_, frame) => input({
    move_y: 1,
    dash_pressed: frame === 0,
  })),
  verify: verifyDownwardCornerCorrection,
})

function verifyDownwardCornerCorrection(states: readonly E2EState[]): void {
  const corrected = states.findIndex((state, frame) => {
    const before = states[frame - 1]
    return before !== undefined
      && near(state.pos[0] - before.pos[0], 1)
      && near(state.pos[1] - before.pos[1], 1)
      && near(state.speed[0], 0)
      && near(state.speed[1], 240)
  })
  semanticAssert(corrected >= 0, scenario.name,
    `downward dash did not move right/down around the one-pixel floor overlap: ${JSON.stringify(states.map(pickCore))}`)
}
