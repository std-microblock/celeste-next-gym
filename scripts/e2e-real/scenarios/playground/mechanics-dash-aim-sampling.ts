import { input } from '../../inputs.js'
import { defineScenario } from '../../scenario.js'
import { PLAYGROUND_TARGET } from '../../targets.js'
import type { E2EState } from '../../types.js'
import { field, near, pickCore, semanticAssert } from '../../verify.js'
import { PLAYGROUND_DASHLESS } from '../common-parts.js'

export const mapParts = [PLAYGROUND_DASHLESS] as const

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: 'active',
  tags: [],
  techniqueIds: ['1.13'],
  recording: { primaryFor: ['1.13'], startFrame: 0, endFrame: 16, posterFrame: 6 },
  mapParts,
  name: 'mechanics-dash-aim-sampling',
  initial: { pos: [392, 82], speed: [0, 0], facing: true, dashes: 1 },
  inputs: Array.from({ length: 16 }, (_, frame) => input({
    move_x: frame >= 2 ? -1 : 0,
    dash_pressed: frame === 1,
  })),
  verify: verifyDashAimSampling,
})

function verifyDashAimSampling(states: readonly E2EState[]): void {
  const entered = states.findIndex((state, frame) => frame > 0 && (state.state === 'Dash' || state.state === 2))
  semanticAssert(entered === 2, scenario.name, `Dash did not begin on state frame 2: ${JSON.stringify(states.slice(0, 4).map(pickCore))}`)
  const beginDir = field<readonly number[]>(states[entered], 'DashDir')
  semanticAssert(near(states[entered]?.speed[0], 0) && near(states[entered]?.speed[1], 0)
    && beginDir && near(beginDir[0], 0) && near(beginDir[1], 0), scenario.name,
  `DashBegin did not clear speed and DashDir: ${JSON.stringify({ core: pickCore(states[entered]), beginDir })}`)

  const launched = states.findIndex((state, frame) => frame > entered && (state.state === 'Dash' || state.state === 2)
    && near(state.speed[0], -240) && near(state.speed[1], 0))
  const launchedDir = field<readonly number[]>(states[launched], 'DashDir')
  semanticAssert(launched === 6 && launchedDir && near(launchedDir[0], -1) && near(launchedDir[1], 0)
    && (states[launched]?.facing === false || states[launched]?.facing === 'Left'), scenario.name,
  `DashCoroutine did not sample frame-3 left after freeze: ${JSON.stringify({ launched, core: pickCore(states[launched]), launchedDir })}`)
}
