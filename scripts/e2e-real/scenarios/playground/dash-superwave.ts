import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import { PLAYGROUND_BASE } from '../common-parts.js'
import type { E2EState } from '../../types.js'
import { field, near, pickCore, semanticAssert } from '../../verify.js'

export const mapParts = [PLAYGROUND_BASE] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: [],
  mapParts,
  name: 'dash-superwave',
    initial: { pos: [240, 496], speed: [0, 0] },
    inputs: Array.from({ length: 30 }, (_, frame) => input({
      move_x: frame <= 10 ? 1 : -1,
      move_y: frame >= 11 ? 1 : 0,
      jump_pressed: frame === 10 || frame === 26,
      jump_held: frame === 10 || frame === 26,
      dash_pressed: frame === 0 || frame === 11,
    })),
    verify: verifySuperwave,
})


function verifySuperwave(states: readonly E2EState[]): void {
  const extended = states[11]
  const landing = states[22]
  const reverse = states[27]
  semanticAssert(extended && near(extended.speed[0], 260) && near(extended.speed[1], -105) && extended.dashes >= 1, 'dash-superwave', 'extended Super keyframe is missing')
  semanticAssert(landing && landing.on_ground && landing.ducking && landing.speed[0] < -200, 'dash-superwave', 'reverse down-diagonal landing keyframe is missing')
  semanticAssert(reverse && near(reverse.speed[0], -325) && near(reverse.speed[1], -52.5) && reverse.dashes >= 1, 'dash-superwave', 'reverse Hyper keyframe is missing')
}
