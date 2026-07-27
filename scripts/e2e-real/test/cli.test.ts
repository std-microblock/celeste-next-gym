import assert from 'node:assert/strict'
import { fileURLToPath } from 'node:url'
import { it } from 'node:test'

import { main } from '../cli.js'

it('rejects unknown scenario selection before invoking runtime side effects', async () => {
  let runtimeCalled = false
  await assert.rejects(() => main(
    ['--scenario', 'does-not-exist'],
    {},
    'D:\\repo',
    {
      run: async () => {
        runtimeCalled = true
        return { health: {}, scenarios: [] }
      },
    },
  ), /unknown E2E scenarios/)
  assert.equal(runtimeCalled, false)
})

it('builds the complete recording plan before any lifecycle and keeps ordinary E2E non-recording', async () => {
  const repoRoot = fileURLToPath(new URL('../../..', import.meta.url))
  let ordinaryCalls = 0
  let recordingCalls = 0
  await main(['--scenario', 'dash'], {}, repoRoot, {
    run: async () => { ordinaryCalls++; return { health: {}, scenarios: [] } },
    runRecording: async () => { recordingCalls++; return { recording: true, scenarioCount: 0, techniqueCount: 0, targets: [] } },
  })
  assert.equal(ordinaryCalls, 1)
  assert.equal(recordingCalls, 0)

  await assert.rejects(() => main(['--record-all'], {}, repoRoot, {
    run: async () => { ordinaryCalls++; return { health: {}, scenarios: [] } },
    runRecording: async () => { recordingCalls++; return { recording: true, scenarioCount: 0, techniqueCount: 0, targets: [] } },
    planRecording: () => { throw new Error('incomplete primary coverage') },
  }), /incomplete primary coverage/)
  assert.equal(ordinaryCalls, 1)
  assert.equal(recordingCalls, 0)
})
