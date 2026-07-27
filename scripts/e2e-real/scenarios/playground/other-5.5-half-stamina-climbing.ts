import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { near, pickCore, semanticAssert } from '../../verify.js'
import { PLAYGROUND_OTHER_HALF_STAMINA } from '../common-parts.js'

export const mapParts = [PLAYGROUND_OTHER_HALF_STAMINA] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['5.5'],
  recording: { primaryFor: ['5.5'], startFrame: 0, endFrame: 3 },
  mapParts,
  name: 'other-5.5-half-stamina-climbing',
  initial: { pos: [620, 400], speed: [0, 0], state: 'Climb', facing: 'Right', stamina: 80 },
  inputs: [
    input({ jump_pressed: true, jump_held: true, grab_held: true }),
    input({ move_x: -1, jump_held: true }),
    input({ move_x: 1, jump_pressed: true, jump_held: true, grab_held: true }),
  ],
  verify: verifyHalfStaminaClimbing,
})

function verifyHalfStaminaClimbing(states: readonly E2EState[]): void {
  semanticAssert(states.length === 4, 'other-5.5-half-stamina-climbing', `expected four states: ${JSON.stringify(states.map(pickCore))}`)
  semanticAssert(near(states[1]?.stamina ?? 0, 52.5), 'other-5.5-half-stamina-climbing', `first climb jump did not spend 27.5 stamina: ${JSON.stringify(states.map(pickCore))}`)
  semanticAssert(near(states[3]?.stamina ?? 0, 52.5), 'other-5.5-half-stamina-climbing', `wallboost refund and close-wall climb jump did not net one 27.5 cost: ${JSON.stringify(states.map(pickCore))}`)
  semanticAssert(near(states[3]?.speed[1] ?? 0, -105), 'other-5.5-half-stamina-climbing', `second climb jump did not launch at -105: ${JSON.stringify(states.map(pickCore))}`)
  semanticAssert(states[3]?.state === 'Normal' && !states[3]?.on_ground && !states[3]?.ducking && !states[3]?.dead,
    'other-5.5-half-stamina-climbing', `second action did not finish airborne in Normal: ${JSON.stringify(states.map(pickCore))}`)
}
