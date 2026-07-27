import assert from 'node:assert/strict'
import { describe, it } from 'node:test'

import { buildRegistry, selectScenarios } from '../registry.js'
import { scenarios } from '../scenarios/index.js'

describe('production scenario registry', () => {
  const registry = buildRegistry(scenarios)

  it('derives all target and status counts from explicit indexes', () => {
    assert.equal(registry.scenarios.length, 142)
    assert.equal(registry.byTarget.get('playground')?.length, 100)
    assert.equal(registry.byTarget.get('area-1')?.length, 36)
    assert.equal(registry.byTarget.get('area-2')?.length, 5)
    assert.equal(registry.byTarget.get('area-4')?.length, 1)
    assert.deepEqual(registry.counts, { active: 134, candidate: 8 })
  })

  it('keeps evidence-less entity scenarios as opt-in candidates', () => {
    const candidates = registry.scenarios
      .filter((scenario) => scenario.status === 'candidate')
      .map((scenario) => scenario.name)
    assert.deepEqual(candidates, [
      'entity-4.10.3-dream-smuggle',
      'entity-4.10.3.1-dream-grab-hyper',
      'entity-4.10.3.2-holdable-dream-hyper',
      'entity-4.10.4-holdable-grabless-dream-hyper',
      'entity-4.15.2-feather-hitbox-preservation',
      'entity-4.6.1-cloud-hyper',
      'entity-4.6.1-cloud-super',
      'entity-4.6.2-cloud-hyper-bunnyhop',
    ])
    assert.equal(selectScenarios(registry, { target: 'playground' }).some((scenario) => scenario.status === 'candidate'), false)
  })

  it('does not invent the absent 2.8.2.1 scenario and wires map parts by target', () => {
    assert.equal(registry.scenarios.some((scenario) => scenario.name.includes('2.8.2.1')), false)
    assert.equal(registry.scenarios.filter((scenario) => scenario.target.kind === 'external').every((scenario) => scenario.mapParts.length === 0), true)
    assert.equal(registry.scenarios.filter((scenario) => scenario.target.kind === 'playground').every((scenario) => scenario.mapParts.length > 0), true)
  })

  it('keeps every dashless cornerboost proof in an independently named map part', () => {
    const techniqueIds = ['3.7.7', '3.7.8', '3.7.9', '3.7.10']
    const parts = techniqueIds.map((techniqueId) => {
      const scenario = registry.scenarios.find((candidate) => candidate.techniqueIds.includes(techniqueId))
      assert.ok(scenario, `missing scenario for ${techniqueId}`)
      assert.equal(scenario.mapParts.length, 1)
      return scenario.mapParts[0]?.id
    })
    assert.equal(new Set(parts).size, techniqueIds.length)
    assert.deepEqual(parts, [
      'tech.3.7.7.eleven-jump',
      'tech.3.7.8.reverse-cornerboost',
      'tech.3.7.9.neutral-reverse-cornerboost',
      'tech.3.7.10.spiked-cornerboost',
    ])
  })

  it('keeps every dashless spike proof in an independently named map part', () => {
    const techniqueIds = ['3.8', '3.8.1', '3.9', '3.10', '3.12.1', '3.13']
    const parts = techniqueIds.map((techniqueId) => {
      const scenario = registry.scenarios.find((candidate) => candidate.techniqueIds.includes(techniqueId))
      assert.ok(scenario, `missing scenario for ${techniqueId}`)
      assert.equal(scenario.mapParts.length, 1)
      return scenario.mapParts[0]?.id
    })
    assert.equal(new Set(parts).size, techniqueIds.length)
    assert.deepEqual(parts, [
      'tech.3.8.spike-climb',
      'tech.3.8.1.narrow-spiked-climb',
      'tech.3.9.spike-clip',
      'tech.3.10.spike-jump',
      'tech.3.12.1.cornerboost-wallboost',
      'tech.3.13.cornerslip',
    ])
  })
})
