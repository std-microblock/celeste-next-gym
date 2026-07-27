import assert from 'node:assert/strict'
import { describe, it } from 'node:test'

import { executeScenario } from '../runtime/scenario-runner.js'
import { reflectedState, testScenario } from './helpers.js'

describe('scenario execution ordering', () => {
  it('validates structure, writes the trace, verifies semantics, then compares Rust', async () => {
    const events: string[] = []
    const scenario = testScenario('ordered', {
      verify: () => { events.push('verify') },
    })
    await executeScenario({
      scenario,
      map: new Uint8Array([1]),
      mapPath: 'map.bin',
      repoRoot: 'D:\\repo',
      skipTransitions: false,
      collectOnly: false,
      dependencies: {
        simulate: async () => { events.push('simulate'); return [reflectedState(), reflectedState({ _frame: 1 })] },
        writeTrace: () => { events.push('trace') },
        compare: () => { events.push('compare') },
      },
    })
    assert.deepEqual(events, ['simulate', 'trace', 'verify', 'compare'])
  })

  it('leaves a trace when semantic verification fails and does not compare', async () => {
    const events: string[] = []
    const scenario = testScenario('failing', { verify: () => { events.push('verify'); throw new Error('guard') } })
    await assert.rejects(() => executeScenario({
      scenario, map: new Uint8Array(), mapPath: 'map.bin', repoRoot: 'D:\\repo',
      skipTransitions: false, collectOnly: false,
      dependencies: {
        simulate: async () => [reflectedState(), reflectedState({ _frame: 1 })],
        writeTrace: () => { events.push('trace') },
        compare: () => { events.push('compare') },
      },
    }), /guard/)
    assert.deepEqual(events, ['trace', 'verify'])
  })

  it('rejects malformed core states before writing a trace', async () => {
    let traceWritten = false
    await assert.rejects(() => executeScenario({
      scenario: testScenario('malformed'), map: new Uint8Array(), mapPath: 'map.bin', repoRoot: 'D:\\repo',
      skipTransitions: false, collectOnly: false,
      dependencies: {
        simulate: async () => [reflectedState(), { ...reflectedState({ _frame: 1 }), speed: [Number.NaN, 0] }],
        writeTrace: () => { traceWritten = true },
        compare: () => undefined,
      },
    }), /invalid position or speed/)
    assert.equal(traceWritten, false)
  })
})
