import assert from 'node:assert/strict'
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
