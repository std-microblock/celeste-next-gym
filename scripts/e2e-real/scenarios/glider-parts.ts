import { defineMapPart } from '../map-parts.js'

const PACKAGE = 'CelesteGymPlayground'
const SID = 'CelesteGymPlayground/Playground'

function gliderPart(id: string, room: Parameters<typeof defineMapPart>[0]['rooms'][number]) {
  return defineMapPart({ id, dependencies: [], package: PACKAGE, sid: SID, rooms: [room] })
}

export const ENTITY_4_20_JELLY_REGRAB_PART = gliderPart('tech.entity-4.20-jelly-regrab', {
  name: 'playground', bounds: [0, 0, 320, 544], spawn: [60, 496], solids: [[0, 496, 320, 48]],
  entities: [{ id: 'tech-4.20-jelly', kind: 'glider', bounds: [56, 486, 8, 10], name: 'glider' }],
})

export const ENTITY_4_22_1_HOLDABLE_STALL_PART = gliderPart('tech.entity-4.22.1-holdable-stall', {
  name: 'playground', bounds: [0, 0, 320, 544], spawn: [80, 320], solids: [[0, 496, 320, 48]],
  entities: [
    { id: 'tech-4.22.1-jelly-a', kind: 'glider', bounds: [76, 310, 8, 10], name: 'glider' },
    { id: 'tech-4.22.1-jelly-b', kind: 'glider', bounds: [76, 390, 8, 10], name: 'glider' },
  ],
})

export const ENTITY_4_22_3_JELLY_NEUTRAL_PART = gliderPart('tech.entity-4.22.3-jelly-neutral', {
  name: 'playground', bounds: [0, 0, 320, 544], spawn: [140, 420],
  solids: [[0, 496, 320, 48], [144, 240, 16, 256]],
  entities: [{ id: 'tech-4.22.3-jelly', kind: 'glider', bounds: [136, 410, 8, 10], name: 'glider' }],
})

export const ENTITY_4_22_4_HOLDABLE_LADDER_PART = gliderPart('tech.entity-4.22.4-holdable-ladder', {
  name: 'playground', bounds: [0, 0, 320, 544], spawn: [96, 400], solids: [[0, 496, 320, 48]],
  entities: [
    { id: 'tech-4.22.4-jelly-a', kind: 'glider', bounds: [92, 390, 8, 10], name: 'glider' },
    { id: 'tech-4.22.4-jelly-b', kind: 'glider', bounds: [92, 430, 8, 10], name: 'glider' },
  ],
})

export const ENTITY_4_23_JELLY_ULTRA_PART = gliderPart('tech.entity-4.23-jelly-ultra', {
  name: 'playground', bounds: [0, 0, 320, 184], spawn: [32, 160], solids: [[0, 160, 320, 24]],
  entities: [{ id: 'tech-4.23-jelly', kind: 'glider', bounds: [64, 150, 8, 10], name: 'glider' }],
})

export const ENTITY_4_26_JELLYVATOR_PART = gliderPart('tech.entity-4.26-jellyvator', {
  name: 'playground', bounds: [0, 0, 320, 544], spawn: [60, 496], solids: [[0, 496, 320, 48]],
  entities: [{ id: 'tech-4.26-jelly', kind: 'glider', bounds: [56, 486, 8, 10], name: 'glider' }],
})

export const ENTITY_4_29_SPRINGBOOST_CANCEL_PART = gliderPart('tech.entity-4.29-springboost-cancel', {
  name: 'playground', bounds: [0, 0, 320, 544], spawn: [100, 496], solids: [[0, 496, 320, 48]],
  entities: [
    { id: 'tech-4.29-jelly', kind: 'glider', bounds: [96, 486, 8, 10], name: 'glider' },
    { id: 'tech-4.29-spring', kind: 'spring', bounds: [128, 490, 16, 6], direction: [0, -1], name: 'spring' },
  ],
})
