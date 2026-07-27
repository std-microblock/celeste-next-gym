import assert from 'node:assert/strict'
import { describe, it } from 'node:test'

import { buildRegistry, selectScenarios } from '../registry.js'
import { scenarios } from '../scenarios/index.js'

describe('production scenario registry', () => {
  const registry = buildRegistry(scenarios)

  it('derives all target and status counts from explicit indexes', () => {
    assert.equal(registry.scenarios.length, 124)
    assert.equal(registry.byTarget.get('playground')?.length, 82)
    assert.equal(registry.byTarget.get('area-1')?.length, 36)
    assert.equal(registry.byTarget.get('area-2')?.length, 5)
    assert.equal(registry.byTarget.get('area-4')?.length, 1)
    assert.deepEqual(registry.counts, { active: 120, candidate: 4 })
  })

  it('keeps evidence-less entity scenarios as opt-in candidates', () => {
    const candidates = registry.scenarios
      .filter((scenario) => scenario.status === 'candidate')
      .map((scenario) => scenario.name)
    assert.deepEqual(candidates, [
      'entity-4.15.2-feather-hitbox-preservation',
      'entity-4.3-bumper-clip',
      'entity-4.4-explosion-boost',
      'entity-4.5-iceball-jump',
    ])
    assert.equal(selectScenarios(registry, { target: 'playground' }).some((scenario) => scenario.status === 'candidate'), false)
  })

  it('does not invent the absent 2.8.2.1 scenario and wires map parts by target', () => {
    assert.equal(registry.scenarios.some((scenario) => scenario.name.includes('2.8.2.1')), false)
    assert.equal(registry.scenarios.filter((scenario) => scenario.target.kind === 'external').every((scenario) => scenario.mapParts.length === 0), true)
    assert.equal(registry.scenarios.filter((scenario) => scenario.target.kind === 'playground').every((scenario) => scenario.mapParts.length > 0), true)
  })
})
