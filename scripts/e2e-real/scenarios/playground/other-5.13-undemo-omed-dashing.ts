import { inputFrames } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { field, near, pickCore, semanticAssert } from '../../verify.js'
import { TECH_OTHER_5_13_UNDEMO_DASHING } from '../common-parts.js'

export const mapParts = [TECH_OTHER_5_13_UNDEMO_DASHING] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['5.13'],
  mapParts,
  name: 'other-5.13-undemo-omed-dashing',
  recording: { primaryFor: ['5.13'], startFrame: 0, endFrame: 10 },
  initial: { pos: [160, 80], speed: [0, 0], dashes: 1 },
  inputs: inputFrames(10, (frame) => ({
    move_x: frame === 0 ? 1 : 0,
    move_y: frame > 0 ? 1 : 0,
    jump_pressed: false,
    jump_held: false,
    dash_pressed: frame === 0,
    crouch_dash_pressed: false,
    grab_held: false,
  })),
  verify: verifyUndemoDashing,
})

function verifyUndemoDashing(states: readonly E2EState[]): void {
  semanticAssert(states[1]?.state === 2 || states[1]?.state === 'Dash', 'other-5.13-undemo-omed-dashing', `dash did not begin on frame one: ${JSON.stringify(states.slice(0, 3).map(pickCore))}`)
  semanticAssert(!states[1]?.ducking, 'other-5.13-undemo-omed-dashing', 'horizontal launch unexpectedly selected the duck collider')
  const redirected = states.find((state, frame) => frame > 1 && (state.state === 2 || state.state === 'Dash') && near(state.speed[0], 0) && near(state.speed[1], 240))
  semanticAssert(redirected !== undefined && !redirected.ducking, 'other-5.13-undemo-omed-dashing', `freeze-time downward aim did not produce a standing downward dash: ${JSON.stringify(states.map(pickCore))}`)
  const dashDir = redirected ? field<readonly number[]>(redirected, 'DashDir') : undefined
  semanticAssert(dashDir !== undefined && near(dashDir[0], 0) && near(dashDir[1], 1), 'other-5.13-undemo-omed-dashing', `redirected DashDir is not down: ${JSON.stringify(dashDir)}`)
  semanticAssert(!states.some((state) => state.dead), 'other-5.13-undemo-omed-dashing', 'player died during undemo candidate')
}
