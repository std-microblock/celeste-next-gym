import assert from 'node:assert/strict'
import { describe, it } from 'node:test'

import { assembleFixturePackage, defineMapPart, dependencyClosure } from '../map-parts.js'
import { COMMON_MAP_PARTS } from '../scenarios/common-parts.js'
import type { MapPart } from '../types.js'

const base = defineMapPart({
  id: 'base', dependencies: [], package: 'Fixture', sid: 'Fixture/Map',
  rooms: [{ name: 'z-room', bounds: [0, 0, 104, 104], spawn: [8, 8], solids: [[8, 16, 8, 8]] }],
})
const entity = defineMapPart({
  id: 'entity', dependencies: ['base'], package: 'Fixture', sid: 'Fixture/Map',
  rooms: [{ name: 'z-room', entities: [
    { id: 'b', kind: 'bumper', bounds: [32, 32, 16, 16] },
    { id: 'a', kind: 'spring', bounds: [16, 16, 8, 8], direction: [0, -1] },
  ] }],
})

describe('map-part exporter', () => {
  it('assembles the full canonical Playground partition', () => {
    const fixture = assembleFixturePackage([...COMMON_MAP_PARTS.values()], COMMON_MAP_PARTS)
    assert.equal(fixture.rooms.length, 2)
    assert.equal(fixture.rooms.reduce((total, room) => total + room.solids.length, 0), 27)
    assert.equal(fixture.rooms.reduce((total, room) => total + room.entities.length, 0), 61)
  })

  it('computes dependency closure and emits canonical stable sorting', () => {
    const catalog = new Map<string, MapPart>([['entity', entity], ['base', base]])
    assert.deepEqual(dependencyClosure([entity], catalog).map((part) => part.id), ['base', 'entity'])
    const fixture = assembleFixturePackage([entity], catalog)
    assert.equal(fixture.formatVersion, 1)
    assert.deepEqual(fixture.rooms[0]?.entities.map((item) => item.id), ['a', 'b'])
    assert.deepEqual(fixture.rooms[0]?.bounds, [0, 0, 104, 104])
  })

  it('rejects unknown dependencies, cycles, and conflicting stable ids', () => {
    assert.throws(() => dependencyClosure([entity], new Map([['entity', entity]])), /unknown/)
    const cycleA = defineMapPart({ ...base, id: 'cycle-a', dependencies: ['cycle-b'] })
    const cycleB = defineMapPart({ ...base, id: 'cycle-b', dependencies: ['cycle-a'] })
    assert.throws(() => dependencyClosure([cycleA], new Map<string, MapPart>([['cycle-a', cycleA], ['cycle-b', cycleB]])), /cycle/)
    const conflict = defineMapPart({
      ...entity,
      id: 'conflict',
      rooms: [{ name: 'z-room', entities: [{ id: 'a', kind: 'bumper', bounds: [90, 90, 8, 8] }] }],
    })
    assert.throws(() => assembleFixturePackage([entity, conflict], new Map<string, MapPart>([
      ['base', base], ['entity', entity], ['conflict', conflict],
    ])), /duplicate map-global fixture entity|conflicting fixture entity/)
  })
})
