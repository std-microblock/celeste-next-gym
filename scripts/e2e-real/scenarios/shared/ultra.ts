import type { E2EState, ScenarioVerifier } from '../../types.js'
import { field, near, pickCore, semanticAssert } from '../../verify.js'

export function ultraLandingFrames(states: readonly E2EState[]): number[] {
  const frames: number[] = []
  for (let frame = 1; frame < states.length; frame++) {
    const before = states[frame - 1]
    const after = states[frame]
    if (before && after && !before.on_ground && after.on_ground && after.ducking && after.speed[0] > before.speed[0]) frames.push(frame)
  }
  return frames
}

export function ultraMultiplierFrames(states: readonly E2EState[]): number[] {
  const frames: number[] = []
  for (let frame = 1; frame < states.length; frame++) {
    const before = states[frame - 1]
    const after = states[frame]
    const beforeDir = field<readonly number[]>(before, 'DashDir')
    const afterDir = field<readonly number[]>(after, 'DashDir')
    if (before && after && beforeDir && afterDir
      && (beforeDir[1] ?? 0) > 0 && near(afterDir[0], Math.sign(beforeDir[0] ?? 0)) && near(afterDir[1], 0)
      && after.on_ground && after.ducking && after.speed[0] > before.speed[0]) frames.push(frame)
  }
  return frames
}

export const verifyUltra: ScenarioVerifier = (states) => {
  const landing = ultraLandingFrames(states).find((frame) => states[frame]?.state === 2)
  semanticAssert(landing !== undefined, 'dash-ultra', 'no in-Dash landing applied the 1.2 multiplier')
  const landed = states[landing]
  semanticAssert(landed, 'dash-ultra', 'landing state is missing')
  const expected = 240 * Math.SQRT1_2 * 1.2
  semanticAssert(near(landed.speed[0], expected) && near(landed.speed[1], 0), 'dash-ultra', `landing speed was ${JSON.stringify(landed.speed)} instead of ${expected}/0`)
  const dashDir = field<readonly number[]>(landed, 'DashDir')
  semanticAssert(dashDir && near(dashDir[0], 1) && near(dashDir[1], 0), 'dash-ultra', `landing did not flatten DashDir to 1/0: ${JSON.stringify(dashDir)}`)
}

export const verifyGroundedUltra: ScenarioVerifier = (states) => {
  semanticAssert(states.some((state) => state.ducking && state.speed[0] >= 359.99), 'dash-grounded-ultra', 'grounded landing never preserved 300 entry speed and applied 1.2')
}

export const verifyDelayedUltra: ScenarioVerifier = (states) => {
  const landing = ultraLandingFrames(states).find((frame) => states[frame]?.state !== 2)
  semanticAssert(landing !== undefined, 'dash-delayed-ultra', `no post-Dash landing applied the delayed 1.2 multiplier: ${JSON.stringify(states.map(pickCore))}`)
  const before = states[landing - 1]
  const after = states[landing]
  semanticAssert(before && after, 'dash-delayed-ultra', 'landing neighbors are missing')
  const expected = Math.max(90, before.speed[0] - 400 * 0.65 / 60) * 1.2
  semanticAssert(near(after.speed[0], expected) && near(after.speed[1], 0), 'dash-delayed-ultra', `post-Dash landing speed was ${JSON.stringify(after.speed)} instead of ${expected}/0`)
}

export const verifyChainedUltras: ScenarioVerifier = (states) => {
  const first = ultraMultiplierFrames(states)[0]
  semanticAssert(first !== undefined, 'dash-chained-ultras', 'first airborne Ultra landing was not observed')
  const firstState = states[first]
  semanticAssert(firstState, 'dash-chained-ultras', 'first Ultra state is missing')
  const second = states.findIndex((state, frame) => frame > first
    && state.state === 2 && state.on_ground && state.ducking && near(state.speed[1], 0)
    && near(state.speed[0], firstState.speed[0] * 1.2))
  semanticAssert(second > first, 'dash-chained-ultras', `second grounded Ultra did not compound ${firstState.speed[0]} by 1.2: ${JSON.stringify(states.map(pickCore))}`)
}
