import { defineMapPart } from '../map-parts.js'

const PACKAGE = 'CelesteGymPlayground'
const SID = 'CelesteGymPlayground/Playground'

export const ENTITY_4_20_THEO_REGRAB_PART = defineMapPart({
  id: 'tech.entity-4.20-theo-regrab',
  dependencies: [],
  package: PACKAGE,
  sid: SID,
  rooms: [{
    name: 'playground',
    bounds: [0, 0, 960, 544],
    spawn: [60, 496],
    solids: [[0, 496, 960, 48]],
    entities: [{ id: 'tech-4.20-theo', kind: 'theo_crystal', bounds: [56, 486, 8, 10], name: 'theoCrystal' }],
  }],
})

export const ENTITY_4_22_NEUTRAL_DROP_PART = defineMapPart({
  id: 'tech.entity-4.22-neutral-drop',
  dependencies: [],
  package: PACKAGE,
  sid: SID,
  rooms: [{
    name: 'playground',
    bounds: [0, 0, 960, 544],
    spawn: [60, 496],
    solids: [[0, 496, 960, 48]],
    entities: [{ id: 'tech-4.22-theo', kind: 'theo_crystal', bounds: [136, 486, 8, 10], name: 'theoCrystal' }],
  }],
})

export const ENTITY_4_22_2_HOLDABLE_CLIMB_PART = defineMapPart({
  id: 'tech.entity-4.22.2-holdable-climb',
  dependencies: [],
  package: PACKAGE,
  sid: SID,
  rooms: [{
    name: 'playground',
    bounds: [0, 0, 960, 544],
    spawn: [60, 496],
    solids: [[0, 496, 960, 48], [224, 240, 16, 256]],
    entities: [{ id: 'tech-4.22.2-theo', kind: 'theo_crystal', bounds: [216, 410, 8, 10], name: 'theoCrystal' }],
  }],
})

export const ENTITY_4_22_3_HOLDABLE_NEUTRAL_PART = defineMapPart({
  id: 'tech.entity-4.22.3-holdable-neutral',
  dependencies: [],
  package: PACKAGE,
  sid: SID,
  rooms: [{
    name: 'playground',
    bounds: [0, 0, 960, 544],
    spawn: [60, 496],
    solids: [[0, 496, 960, 48], [384, 240, 16, 256]],
    entities: [{ id: 'tech-4.22.3-theo', kind: 'theo_crystal', bounds: [376, 410, 8, 10], name: 'theoCrystal' }],
  }],
})
