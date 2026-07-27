import { describe, expect, it } from 'vitest'
import { PLAYGROUND, createInitialState, makeEmptyButtons } from '../model'
import { compareTraces, createWebTrace, initialStateFromTrace, parseTrace } from './trace'

describe('portable frame traces', () => {
  it('exports one state more than inputs and validates the result', () => {
    const initial = createInitialState(PLAYGROUND)
    const next = { ...structuredClone(initial), pos: { x: 65, y: 496 }, on_ground: true }
    const trace = createWebTrace(PLAYGROUND, [makeEmptyButtons()], [initial, next], 1, '2026-01-01T00:00:00.000Z')
    expect(trace.inputs).toHaveLength(1)
    expect(trace.states).toHaveLength(2)
    expect(parseTrace(trace)).toBe(trace)
  })

  it('compares all nine product-gate fields at 0.01 tolerance', () => {
    const initial = createInitialState(PLAYGROUND)
    const expected = createWebTrace(PLAYGROUND, [], [initial], 0)
    const within = structuredClone(expected)
    within.states[0].pos[0] += 0.009
    within.states[0].speed[1] -= 0.009
    expect(compareTraces(within, expected).matched).toBe(true)
    within.states[0].stamina -= 0.011
    const mismatch = compareTraces(within, expected)
    expect(mismatch.matched).toBe(false)
    expect(mismatch.first_mismatch).toBe(0)
  })

  it('rejects incomplete traces', () => {
    const initial = createInitialState(PLAYGROUND)
    const trace = createWebTrace(PLAYGROUND, [], [initial], 0)
    expect(() => parseTrace({ ...trace, states: [] })).toThrow(/inputs \+ 1/)
  })

  it('uses a game trace F0 as the simulator replay checkpoint', () => {
    const state = initialStateFromTrace({
      frame: 0, pos: [80, 120], speed: [90, -10], state: 2, facing: 'Left',
      dashes: 0, stamina: 88, on_ground: true, ducking: true, dead: false,
    }, PLAYGROUND)
    expect(state).toMatchObject({
      pos: { x: 80, y: 120 }, speed: { x: 90, y: -10 }, state: 'Dash',
      facing: false, dashes: 0, stamina: 88, on_ground: true, ducking: true,
    })
  })
})
