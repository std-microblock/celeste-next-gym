import { defineMapPart } from '../map-parts.js'

const PACKAGE = 'CelesteGymPlayground'
const SID = 'CelesteGymPlayground/Playground'
const ROOM = 'playground'

export const TECH_ENTITY_4_11_HOLDABLE_CORE_HYPER = defineMapPart({
  id: 'tech.entity-4.11-holdable-core-hyper',
  dependencies: ['playground.base'],
  package: PACKAGE,
  sid: SID,
  rooms: [{
    name: ROOM,
    entities: [
      { id: 'tech-4.11-bounce-block', kind: 'bounce_block', bounds: [352, 360, 64, 16], name: 'bounceBlock' },
      { id: 'tech-4.11-theo', kind: 'theo_crystal', bounds: [380, 350, 8, 10], name: 'theoCrystal' },
    ],
  }],
})

export const TECH_ENTITY_4_14_HEART_ULTRA = defineMapPart({
  id: 'tech.entity-4.14-heart-ultra',
  dependencies: ['playground.base'],
  package: PACKAGE,
  sid: SID,
  rooms: [{
    name: ROOM,
    entities: [
      { id: 'tech-4.14-heart', kind: 'heart_gem', bounds: [488, 480, 16, 16], name: 'heartGem' },
    ],
  }],
})

export const TECH_ENTITY_4_15_JUMPTHROUGH_CLIP = defineMapPart({
  id: 'tech.entity-4.15-jumpthrough-clip',
  dependencies: ['playground.base'],
  package: PACKAGE,
  sid: SID,
  rooms: [{
    name: ROOM,
    entities: [
      { id: 'tech-4.15-zip-mover', kind: 'zip_mover', bounds: [592, 304, 64, 16], nodes: [[592, 408]], name: 'zipMover' },
      { id: 'tech-4.15-jump-thru', kind: 'jump_thru', bounds: [568, 416, 112, 8], name: 'jumpThru' },
    ],
  }],
})

export const CORE_HEART_SQUISH_MAP_PARTS = [
  TECH_ENTITY_4_11_HOLDABLE_CORE_HYPER,
  TECH_ENTITY_4_14_HEART_ULTRA,
  TECH_ENTITY_4_15_JUMPTHROUGH_CLIP,
] as const
