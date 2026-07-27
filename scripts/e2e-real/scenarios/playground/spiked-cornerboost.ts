import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { field, near, semanticAssert } from '../../verify.js'
import { SPIKED_CORNERBOOST_PART } from '../dashless-cornerboost-parts.js'

export const mapParts = [SPIKED_CORNERBOOST_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: [],
  techniqueIds: ['3.7.10'],
  mapParts,
  name: 'spiked-cornerboost',
  initial: { pos: [235, 246], speed: [90, -30] },
  inputs: Array.from({ length: 20 }, (_, frame) => input({
    move_x: 1,
    jump_pressed: frame === 0,
    jump_held: frame < 12,
    grab_held: frame === 0,
  })),
  verify: verifySpikedCornerboost,
})

function verifySpikedCornerboost(states: readonly E2EState[]): void {
  semanticAssert(states.every((state) => !state.dead), 'spiked-cornerboost', 'upward cornerboost was killed by the top spikes while moving away from their points')
  semanticAssert(near(states[1]?.stamina, 82.5) && near(states[1]?.speed[0], 0), 'spiked-cornerboost', 'cornerboost did not store its first horizontal collision')
  semanticAssert(Number(field(states[1], 'wallSpeedRetained')) > 120, 'spiked-cornerboost', `retained speed was ${String(field(states[1], 'wallSpeedRetained'))}`)
}
