import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { field, near, pickCore, semanticAssert } from '../../verify.js'
import { PLAYGROUND_OTHER_KERMIT } from '../common-parts.js'

export const mapParts = [PLAYGROUND_OTHER_KERMIT] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['5.6'],
  recording: { primaryFor: ['5.6'], startFrame: 0, endFrame: 48 },
  mapParts,
  name: 'other-5.6-kermit-dash',
  initial: { pos: [630, 12], speed: [0, 0], dashes: 1 },
  inputs: Array.from({ length: 48 }, (_, frame) => input({ move_y: -1, dash_pressed: frame === 0 })),
  verify: verifyKermitDash,
})

function verifyKermitDash(states: readonly E2EState[]): void {
  const entered = states.findIndex((state, frame) => frame > 0 && (state.state === 'Normal' || state.state === 0) && near(state.speed[1], -105))
  semanticAssert(entered > 0, 'other-5.6-kermit-dash', `upward transition did not cancel Dash into Normal: ${JSON.stringify(states.slice(0, 6).map(pickCore))}`)
  const dashDir = field<readonly number[]>(states[entered]!, 'DashDir')
  const dashAttack = field<number>(states[entered]!, 'dashAttackTimer')
  semanticAssert(Array.isArray(dashDir) && near(Number(dashDir[0]), 0) && near(Number(dashDir[1]), -1), 'other-5.6-kermit-dash', `transition lost upward dashDir: ${JSON.stringify(dashDir)}`)
  semanticAssert(typeof dashAttack === 'number' && dashAttack > 0, 'other-5.6-kermit-dash', `transition lost dash attack timer: ${JSON.stringify(dashAttack)}`)
  const hit = states.findIndex((state, frame) => frame > entered && (state.state === 'StarFly' || state.state === 19))
  semanticAssert(hit > entered, 'other-5.6-kermit-dash', `preserved dash attack did not break the upper-room feather shield: ${JSON.stringify(states.slice(-8).map(pickCore))}`)
  semanticAssert(field(states[hit]!, 'dashAttackTimer') === 0 && !states[hit]?.dead, 'other-5.6-kermit-dash', 'shield interaction did not consume dash attack safely')
}
