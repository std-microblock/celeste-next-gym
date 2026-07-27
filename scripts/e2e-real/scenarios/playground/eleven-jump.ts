import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { field, near, semanticAssert } from '../../verify.js'
import { ELEVEN_JUMP_PART } from '../dashless-cornerboost-parts.js'

export const mapParts = [ELEVEN_JUMP_PART] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'candidate',
  tags: [],
  techniqueIds: ['3.7.7'],
  mapParts,
  name: 'eleven-jump',
  initial: { pos: [956, 126], speed: [160, -30], stamina: 20 },
  inputs: Array.from({ length: 120 }, (_, frame) => input({
    move_x: 1,
    jump_pressed: frame === 37 || frame === 42 || frame === 43,
    jump_held: frame >= 37 && frame < 60,
    grab_held: frame >= 37 && frame <= 43,
  })),
  verify: verifyElevenJump,
})

function verifyElevenJump(states: readonly E2EState[]): void {
  const completed = states.findIndex((state, frame) => frame > 0 && state.dashes >= 1 && near(state.stamina, 110))
  semanticAssert(completed === 41, 'eleven-jump', `room transition completed at state ${completed} instead of 41`)
  semanticAssert(near(states[42]?.stamina, 82.5) && near(states[43]?.stamina, 55) && near(states[44]?.stamina, 27.5), 'eleven-jump', 'three buffered climb jumps did not execute immediately after the transition')
  semanticAssert(Number(field(states[44], 'wallSpeedRetained')) > 190, 'eleven-jump', `retained speed was ${String(field(states[44], 'wallSpeedRetained'))}`)
  semanticAssert(states.some((state) => state.on_ground && state.pos[0] >= 1052), 'eleven-jump', 'player did not land across the eleven-tile gap')
}
