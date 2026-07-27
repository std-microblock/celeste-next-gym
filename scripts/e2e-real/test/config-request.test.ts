import assert from 'node:assert/strict'
import { describe, it } from 'node:test'

import { parseConfig } from '../config.js'
import { input } from '../inputs.js'
import { createRequest } from '../request.js'
import { AREA_2_TARGET } from '../targets.js'
import { testScenario } from './helpers.js'

describe('configuration and requests', () => {
  it('preserves environment selection and playground feature switches', () => {
    const config = parseConfig([], {
      E2E_AREA_ID: '2',
      E2E_SCENARIOS: 'dreamdash, entity-4.9-dream-grab',
      E2E_PLAYGROUND_BUMPER: '0',
      E2E_SKIP_TRANSITIONS: '1',
    }, 'D:\\repo')
    assert.equal(config.target, 'area-2')
    assert.equal(config.skipTransitions, true)
    assert.deepEqual([...config.requestedScenarios], ['dreamdash', 'entity-4.9-dream-grab'])
    assert.equal(config.disabledTags.has('feature:bumper'), true)
  })

  it('rejects malformed flags, area ids, and target conflicts', () => {
    assert.throws(() => parseConfig(['--wat'], {}, 'D:\\repo'), /unknown/)
    assert.throws(() => parseConfig([], { E2E_AREA_ID: '2oops' }, 'D:\\repo'), /non-negative integer/)
    assert.throws(() => parseConfig(['--target', 'area-4'], { E2E_AREA_ID: '2' }, 'D:\\repo'), /conflicts/)
    assert.throws(() => parseConfig(['--record', '--record-all', '--scenario', 'dash'], {}, 'D:\\repo'), /mutually exclusive/)
    assert.throws(() => parseConfig(['--record'], {}, 'D:\\repo'), /exactly one/)
    assert.throws(() => parseConfig(['--record-all', '--target', 'area-1'], {}, 'D:\\repo'), /cannot be constrained/)
  })

  it('derives area and default map from a CLI target', () => {
    const config = parseConfig(['--target', 'area-2'], {}, 'D:\\repo')
    assert.equal(config.areaId, 2)
    assert.equal(config.mapFile, '2-OldSite.bin')
  })

  it('builds canonical simulate requests without mutating defaults', () => {
    const scenario = testScenario('dream', {
      target: AREA_2_TARGET,
      initial: { pos: [10, 20], dashes: 0 },
      inputs: [input({ dash_pressed: true })],
    })
    const request = createRequest({ scenario, map: new Uint8Array([1, 2]), room: 'room', skipTransitions: true })
    assert.deepEqual(request.initial_snapshot.pos, [10, 20])
    assert.equal(request.initial_snapshot.dashes, 0)
    assert.equal(request.initial_snapshot.can_dream_dash, true)
    assert.equal(request.dream_dash, true)
    assert.equal(request.frames, 1)
    assert.equal(request.room, 'room')
    const captured = createRequest({ scenario, map: new Uint8Array([1]), skipTransitions: false, captureToken: 'x'.repeat(32) })
    assert.equal(captured.capture_token, 'x'.repeat(32))
  })
})
