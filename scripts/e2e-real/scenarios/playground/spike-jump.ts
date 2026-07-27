import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { near, semanticAssert } from '../../verify.js'
import { SPIKE_JUMP_PART } from '../dashless-spike-parts.js'

export const mapParts = [SPIKE_JUMP_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: ['feature:zip-mover'],
  techniqueIds: ['3.10'],
  mapParts,
  name: 'spike-jump',
  recording: { primaryFor: ['3.10'], startFrame: 0, endFrame: 35 },
  initial: { pos: [48, 120], speed: [0, 0], on_ground: true },
  inputs: Array.from({ length: 35 }, (_, frame) => input({
    jump_pressed: frame === 27,
    jump_held: frame >= 27,
  })),
  verify: verifySpikeJump,
})

function verifySpikeJump(states: readonly E2EState[]): void {
  semanticAssert(states.every((state) => !state.dead), 'spike-jump', 'the post-carry spike jump died before leaving the spike surface')
  semanticAssert(states[27]?.on_ground === true, 'spike-jump', 'ZipMover did not carry the player onto a legal grounded frame after the prior spike check')
  semanticAssert(near(states[28]?.speed[1], -105) && states[28]?.on_ground === false, 'spike-jump', `jump frame was ${JSON.stringify(states[28]?.speed)}`)
}
