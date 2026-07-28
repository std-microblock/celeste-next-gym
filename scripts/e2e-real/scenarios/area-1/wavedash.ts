import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { AREA_1_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { near, semanticAssert } from '../../verify.js'

export const mapParts = [] as const

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['2.4'],
  recording: { primaryFor: ['2.4'], startFrame: 0, endFrame: 18 },
  mapParts,
  name: 'wavedash',
  initial: { pos: [70, 114], speed: [0, 0] },
  inputs: Array.from({ length: 18 }, (_, frame) => input({
    move_x: 1,
    move_y: frame <= 9 ? 1 : 0,
    jump_pressed: frame === 9,
    jump_held: frame >= 9 && frame < 16,
    dash_pressed: frame === 0,
  })),
  verify: verifyWavedashMinimumHeight,
})

function verifyWavedashMinimumHeight(states: readonly E2EState[]): void {
  const landing = states[10]
  const wavedash = states[11]
  semanticAssert(landing?.state === 2 && landing.on_ground && landing.ducking && landing.dashes === 0,
    'wavedash', `fourteen-pixel landing did not preserve the diagonal Dash collision window: ${JSON.stringify(landing)}`)
  semanticAssert(wavedash?.state === 0 && near(wavedash.speed[0], 325) && near(wavedash.speed[1], -52.5) && wavedash.dashes === 1,
    'wavedash', `buffered landing jump did not become a refilled Hyper: ${JSON.stringify(wavedash)}`)
}
