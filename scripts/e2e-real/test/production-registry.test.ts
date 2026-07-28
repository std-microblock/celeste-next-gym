import assert from 'node:assert/strict'
import { describe, it } from 'node:test'

import { buildRegistry, selectScenarios } from '../registry.js'
import { scenarios } from '../scenarios/index.js'

describe('production scenario registry', () => {
  const registry = buildRegistry(scenarios)

  it('derives all target and status counts from explicit indexes', () => {
    assert.equal(registry.scenarios.length, 148)
    assert.equal(registry.byTarget.get('playground')?.length, 106)
    assert.equal(registry.byTarget.get('area-1')?.length, 36)
    assert.equal(registry.byTarget.get('area-2')?.length, 5)
    assert.equal(registry.byTarget.get('area-4')?.length, 1)
    assert.deepEqual(registry.counts, { active: 140, candidate: 8 })
  })

  it('keeps evidence-less scenarios as opt-in candidates', () => {
    const candidates = registry.scenarios
      .filter((scenario) => scenario.status === 'candidate')
      .map((scenario) => scenario.name)
    assert.deepEqual(candidates, [
      'entity-4.10.3.2-holdable-dream-hyper',
      'entity-4.10.4-holdable-grabless-dream-hyper',
      'entity-4.15.2-feather-hitbox-preservation',
      'entity-4.20-theo-regrab',
      'entity-4.22-neutral-drop',
      'entity-4.22.2-holdable-climb',
      'entity-4.22.3-holdable-neutral-jump',
      'entity-4.6.2-cloud-hyper-bunnyhop',
    ])
    assert.equal(selectScenarios(registry, { target: 'playground' }).some((scenario) => scenario.status === 'candidate'), false)
  })

  it('links the 2.8.2.1 proof and control scenarios and wires map parts by target', () => {
    assert.equal(registry.scenarios.filter((scenario) => scenario.techniqueIds.includes('2.8.2.1')).length, 2)
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

  it('keeps grounded ultra cancel in its own Theo-only map part', () => {
    const linked = registry.scenarios.filter((scenario) => scenario.techniqueIds.includes('2.8.2.1'))
    for (const scenario of linked) {
      assert.equal(scenario.mapParts.length, 1)
      assert.equal(scenario.mapParts[0]?.id, 'tech.2.8.2.1.grounded-ultra-cancel')
      assert.deepEqual(scenario.mapParts[0]?.dependencies, [])
    }
  })
})
