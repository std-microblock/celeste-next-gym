import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { field, near, semanticAssert } from '../../verify.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['2.10'],
  mapParts,
  name: 'delayed-wallbounce',
  initial: { pos: [140, 112], speed: [0, 0] },
  inputs: Array.from({ length: 24 }, (_, frame) => input({
    move_y: -1,
    jump_pressed: frame === 17,
    jump_held: frame >= 17 && frame < 23,
    dash_pressed: frame === 0,
  })),
  verify: verifyDelayedWallbounce,
})

function verifyDelayedWallbounce(states: readonly E2EState[]): void {
  const beforeJump = states[17]
  const launch = states[18]
  semanticAssert(beforeJump?.state === 0 && Number(field(beforeJump, 'dashAttackTimer')) > 0,
    'delayed-wallbounce', 'jump did not occur in Normal with a lingering dash attack')
  semanticAssert(launch?.state === 0 && near(launch.speed[0], 170) && near(launch.speed[1], -160),
    'delayed-wallbounce', `lingering up-dash produced ${JSON.stringify(launch?.speed)}`)
  semanticAssert(Number(field(launch, 'forceMoveXTimer')) === 0,
    'delayed-wallbounce', `SuperWallJump unexpectedly forced horizontal input: ${String(field(launch, 'forceMoveXTimer'))}`)
}
