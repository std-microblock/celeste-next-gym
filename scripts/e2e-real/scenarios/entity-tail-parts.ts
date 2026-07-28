import { defineMapPart } from '../map-parts.js'

const PACKAGE = 'CelesteGymPlayground'
const SID = 'CelesteGymPlayground/Playground'

export const ENTITY_4_23_THEO_ULTRA_PART = defineMapPart({
  id: 'tech.entity-4.23-theo-ultra', dependencies: [], package: PACKAGE, sid: SID,
  rooms: [{
    name: 'playground', bounds: [0, 0, 320, 184], spawn: [32, 160],
    solids: [[0, 160, 320, 24]],
    entities: [{ id: 'tech-4.23-theo', kind: 'theo_crystal', bounds: [64, 150, 8, 10], name: 'theoCrystal' }],
  }],
})

export const ENTITY_4_24_BUMPER_SMUGGLE_PART = defineMapPart({
  id: 'tech.entity-4.24-bumper-smuggle', dependencies: [], package: PACKAGE, sid: SID,
  rooms: [{
    name: 'playground', bounds: [0, 0, 320, 544], spawn: [100, 496],
    solids: [[0, 496, 320, 48]],
    entities: [
      { id: 'tech-4.24-theo', kind: 'theo_crystal', bounds: [96, 486, 8, 10], name: 'theoCrystal' },
      { id: 'tech-4.24-bumper', kind: 'bumper', bounds: [120, 480, 24, 24], name: 'bigSpinner' },
    ],
  }],
})

export const ENTITY_4_25_THROWABLE_BACKBOOST_PART = defineMapPart({
  id: 'tech.entity-4.25-throwable-backboost', dependencies: [], package: PACKAGE, sid: SID,
  rooms: [{
    name: 'playground', bounds: [0, 0, 320, 544], spawn: [100, 496],
    solids: [[0, 496, 320, 48]],
    entities: [{ id: 'tech-4.25-theo', kind: 'theo_crystal', bounds: [96, 486, 8, 10], name: 'theoCrystal' }],
  }],
})

export const ENTITY_4_26_THEOVATOR_PART = defineMapPart({
  id: 'tech.entity-4.26-theovator', dependencies: [], package: PACKAGE, sid: SID,
  rooms: [{
    name: 'playground', bounds: [0, 0, 320, 544], spawn: [60, 496],
    solids: [[0, 496, 320, 48]],
    entities: [{ id: 'tech-4.26-theo', kind: 'theo_crystal', bounds: [56, 486, 8, 10], name: 'theoCrystal' }],
  }],
})

export const ENTITY_4_27_WATERBOOST_PART = defineMapPart({
  id: 'tech.entity-4.27-waterboost', dependencies: [], package: PACKAGE, sid: SID,
  rooms: [{
    name: 'playground', bounds: [0, 0, 320, 184], spawn: [136, 68],
    solids: [[0, 136, 320, 48]],
    entities: [{ id: 'tech-4.27-water', kind: 'water', bounds: [80, 56, 112, 80], name: 'water' }],
  }],
})
