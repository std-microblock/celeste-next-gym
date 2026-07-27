import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { field, near, semanticAssert } from '../../verify.js'
import { COBWOB_PART } from '../dashless-spike-parts.js'

export const mapParts = [COBWOB_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['3.12.1'],
  mapParts,
  name: 'cornerboost-wallboost',
  recording: { primaryFor: ['3.12.1'], startFrame: 0, endFrame: 12 },
  initial: { pos: [35, 46], speed: [160, -30], facing: 'Right' },
  inputs: Array.from({ length: 12 }, (_, frame) => input({
    move_x: frame === 0 ? 0 : -1,
    jump_pressed: frame === 0,
    jump_held: frame < 10,
    grab_held: frame === 0,
  })),
  verify: verifyCobwob,
})

function verifyCobwob(states: readonly E2EState[]): void {
  semanticAssert((Number(field(states[1], 'wallSpeedRetained')) > 140) && Number(field(states[1], 'wallBoostTimer')) > 0.19, 'cornerboost-wallboost', 'neutral cornerboost did not retain speed and open the wallboost window')
  semanticAssert(near(states[1]?.stamina, 82.5) && near(states[3]?.stamina, 110), 'cornerboost-wallboost', 'wallboost did not refund the climb-jump stamina')
  semanticAssert((states[3]?.speed[0] ?? 0) < -120 && (states[3]?.speed[0] ?? 0) > -130, 'cornerboost-wallboost', `conversion speed was ${states[3]?.speed[0]}`)
}
