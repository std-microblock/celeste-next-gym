import assert from 'node:assert/strict'
import test from 'node:test'

import { compareTraces, validateTrace, type FrameTrace } from '../src/trace.js'

const trace = (): FrameTrace => ({
  format: 'celeste-next-gym-trace', version: 1, source: 'web',
  recorded_at: '2026-01-01T00:00:00.000Z',
  map: { sid: 'CelesteGymPlayground/Playground', room: 'playground', binary: 'maps/CelesteGymPlayground/Playground.bin' },
  inputs: [],
  states: [{ frame: 0, pos: [64, 496], speed: [0, 0], state: 'Normal', facing: true, dashes: 1, stamina: 110, on_ground: false, ducking: false, dead: false }],
})

test('validates the shared game/web trace envelope', () => {
  assert.equal(validateTrace(trace()).states.length, 1)
  assert.throws(() => validateTrace({ ...trace(), states: [] }), /inputs length \+ 1/)
})

test('compares the complete nine-field E2E gate', () => {
  const expected = trace()
  const actual = structuredClone(expected)
  ;(actual.states[0] as { stamina: number }).stamina = 109.98
  const result = compareTraces(actual, expected)
  assert.equal(result.matched, false)
  assert.equal(result.first_mismatch, 0)
  assert.deepEqual(result.differing_fields, ['stamina'])
})
