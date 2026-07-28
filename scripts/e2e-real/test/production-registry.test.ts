import assert from 'node:assert/strict'
import { describe, it } from 'node:test'

import { buildRegistry, selectScenarios } from '../registry.js'
import { scenarios } from '../scenarios/index.js'

describe('production scenario registry', () => {
  const registry = buildRegistry(scenarios)

  it('derives all target and status counts from explicit indexes', () => {
    assert.equal(registry.scenarios.length, 176)
    assert.equal(registry.byTarget.get('playground')?.length, 134)
    assert.equal(registry.byTarget.get('area-1')?.length, 36)
    assert.equal(registry.byTarget.get('area-2')?.length, 5)
    assert.equal(registry.byTarget.get('area-4')?.length, 1)
    assert.deepEqual(registry.counts, { active: 152, candidate: 24 })
  })

  it('keeps evidence-less scenarios as opt-in candidates', () => {
    const candidates = registry.scenarios
      .filter((scenario) => scenario.status === 'candidate')
      .map((scenario) => scenario.name)
    assert.deepEqual(candidates, [
      'entity-4.10.3.2-holdable-dream-hyper',
      'entity-4.10.4-holdable-grabless-dream-hyper',
      'entity-4.14-heart-ultra',
      'entity-4.15-jumpthrough-clip',
      'entity-4.18.2.1-cassoosted-fuper',
      'entity-4.18.3-core-block-entity-displacement',
      'entity-4.19-seeker-bounce',
      'entity-4.22.1-holdable-stall',
      'entity-4.22.2-holdable-climb',
      'entity-4.22.3-holdable-neutral-jump',
      'entity-4.22.3-jelly-neutral-jump',
      'entity-4.22.4-holdable-laddering',
      'entity-4.23-jelly-ultra',
      'entity-4.23-theo-ultra',
      'entity-4.24-bumper-holdable-dash-smuggle',
      'entity-4.26-jellyvator',
      'entity-4.26-theovator',
      'entity-4.28-koral-clip',
      'entity-4.29-springboost-cancel',
      'entity-4.6.2-cloud-hyper-bunnyhop',
      'other-5.10-spinner-stunning',
      'other-5.11-spinner-freeze',
      'other-5.3-cassette-raise',
      'other-5.9-screen-transition-cassette-offset',
    ])
    assert.equal(selectScenarios(registry, { target: 'playground' }).some((scenario) => scenario.status === 'candidate'), false)
  })

  it('keeps Theo regrab and slash proofs in independently named map parts', () => {
    const techniqueIds = ['4.20', '4.21']
    const parts = techniqueIds.map((techniqueId) => {
      const scenario = registry.scenarios.find((candidate) => candidate.techniqueIds.includes(techniqueId) && !candidate.tags.includes('feature:glider'))
      assert.ok(scenario, `missing scenario for ${techniqueId}`)
      assert.equal(scenario.mapParts.length, 1)
      assert.equal(scenario.mapParts[0]?.dependencies.length, 0)
      return scenario.mapParts[0]?.id
    })
    assert.deepEqual(parts, ['tech.entity-4.20-theo-regrab', 'tech.entity-4.21-holdable-slash'])
  })

  it('keeps the dynamic Seeker proof in its own map part', () => {
    const seeker = registry.scenarios.find((candidate) => candidate.techniqueIds.includes('4.19'))
    assert.ok(seeker)
    assert.equal(seeker.mapParts.length, 1)
    assert.equal(seeker.mapParts[0]?.id, 'tech.entity-4.19-seeker-bounce')
    assert.deepEqual(seeker.mapParts[0]?.dependencies, [])
  })

  it('keeps the TempleGate squish proof in its own map part', () => {
    const koral = registry.scenarios.find((candidate) => candidate.techniqueIds.includes('4.28'))
    assert.ok(koral)
    assert.equal(koral.mapParts.length, 1)
    assert.equal(koral.mapParts[0]?.id, 'tech.entity-4.28-koral-clip')
    assert.deepEqual(koral.mapParts[0]?.dependencies, [])
  })

  it('keeps every cassette and spinner audit in an independently named map part', () => {
    const techniqueIds = ['4.18.2', '4.18.2.1', '5.3', '5.9', '5.10', '5.11']
    const parts = techniqueIds.map((techniqueId) => {
      const scenario = registry.scenarios.find((candidate) => candidate.techniqueIds.includes(techniqueId))
      assert.ok(scenario, `missing scenario for ${techniqueId}`)
      assert.equal(scenario.mapParts.length, 1)
      return scenario.mapParts[0]?.id
    })
    assert.equal(new Set(parts).size, techniqueIds.length)
    assert.deepEqual(parts, [
      'tech.entity-4.18.2-reform-boost-cassette-boost',
      'tech.entity-4.18.2.1-cassoosted-fuper',
      'tech.other-5.3-cassette-raise',
      'tech.other-5.9-screen-transition-cassette-offset',
      'tech.other-5.10-spinner-stunning',
      'tech.other-5.11-spinner-freeze',
    ])
  })

  it('keeps every reform proof in an independently named map part', () => {
    const techniqueIds = ['4.17', '4.18', '4.18.1', '4.18.3']
    const parts = techniqueIds.map((techniqueId) => {
      const scenario = registry.scenarios.find((candidate) => candidate.techniqueIds.includes(techniqueId))
      assert.ok(scenario, `missing scenario for ${techniqueId}`)
      assert.equal(scenario.mapParts.length, 1)
      return scenario.mapParts[0]?.id
    })
    assert.deepEqual(parts, [
      'tech.entity-4.17-moon-boost',
      'tech.entity-4.18-reform-tech',
      'tech.entity-4.18.1-reform-kick',
      'tech.entity-4.18.3-core-block-entity-displacement',
    ])
    assert.equal(new Set(parts).size, techniqueIds.length)
  })

  it('links the 2.8.2.1 proof and control scenarios and wires map parts by target', () => {
    assert.equal(registry.scenarios.filter((scenario) => scenario.techniqueIds.includes('2.8.2.1')).length, 2)
    assert.equal(registry.scenarios.filter((scenario) => scenario.target.kind === 'external').every((scenario) => scenario.mapParts.length === 0), true)
    assert.equal(registry.scenarios.filter((scenario) => scenario.target.kind === 'playground').every((scenario) => scenario.mapParts.length > 0), true)
    assert.equal(registry.byTarget.get('area-2')?.every((scenario) => scenario.room === '1'), true)
    assert.equal(registry.byTarget.get('area-4')?.every((scenario) => scenario.room === 'a-02'), true)
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

  it('keeps every feather proof in an independently named map part', () => {
    const techniqueIds = ['4.12', '4.13', '4.15.1', '4.15.2']
    const parts = techniqueIds.map((techniqueId) => {
      const scenario = registry.scenarios.find((candidate) => candidate.techniqueIds.includes(techniqueId))
      assert.ok(scenario, `missing scenario for ${techniqueId}`)
      assert.equal(scenario.mapParts.length, 1)
      return scenario.mapParts[0]?.id
    })
    assert.equal(new Set(parts).size, techniqueIds.length)
    assert.deepEqual(parts, [
      'tech.entity-4.12-featherboost',
      'tech.entity-4.13-feather-super',
      'tech.entity-4.15.1-feather-clip',
      'tech.entity-4.15.2-hitbox-preservation',
    ])
  })

  it('keeps every entity-tail proof in an independently named map part', () => {
    const techniqueIds = ['4.23', '4.24', '4.25', '4.26', '4.27']
    const parts = techniqueIds.map((techniqueId) => {
      const scenario = registry.scenarios.find((candidate) => candidate.techniqueIds.includes(techniqueId) && !candidate.tags.includes('feature:glider'))
      assert.ok(scenario, `missing scenario for ${techniqueId}`)
      assert.equal(scenario.mapParts.length, 1)
      return scenario.mapParts[0]?.id
    })
    assert.equal(new Set(parts).size, techniqueIds.length)
    assert.deepEqual(parts, [
      'tech.entity-4.23-theo-ultra',
      'tech.entity-4.24-bumper-smuggle',
      'tech.entity-4.25-throwable-backboost',
      'tech.entity-4.26-theovator',
      'tech.entity-4.27-waterboost',
    ])
  })

  it('keeps every Glider proof variant in an independently named map part', () => {
    const names = [
      'entity-4.20-jelly-regrab',
      'entity-4.22.1-holdable-stall',
      'entity-4.22.3-jelly-neutral-jump',
      'entity-4.22.4-holdable-laddering',
      'entity-4.23-jelly-ultra',
      'entity-4.26-jellyvator',
      'entity-4.29-springboost-cancel',
    ]
    const parts = names.map((name) => {
      const scenario = registry.byName.get(name)
      assert.ok(scenario, `missing scenario ${name}`)
      assert.equal(scenario.mapParts.length, 1)
      assert.equal(scenario.mapParts[0]?.dependencies.length, 0)
      return scenario.mapParts[0]?.id
    })
    assert.equal(new Set(parts).size, names.length)
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
