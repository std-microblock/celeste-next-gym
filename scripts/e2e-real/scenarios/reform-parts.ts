import { defineMapPart } from '../map-parts.js'
import type { MapPart } from '../types.js'

const PACKAGE = 'CelesteGymPlayground'
const SID = 'CelesteGymPlayground/Playground'
const ROOM = 'playground'

function reformPart(
  id: string,
  rooms: MapPart['rooms'],
): MapPart {
  return defineMapPart({
    id,
    dependencies: ['playground.base'],
    package: PACKAGE,
    sid: SID,
    rooms,
  })
}

export const TECH_ENTITY_4_17_MOON_BOOST = reformPart('tech.entity-4.17-moon-boost', [{
  name: ROOM,
  entities: [
    { id: 'tech-4.17-move-block', kind: 'move_block', bounds: [64, 320, 32, 16], direction: [1, 0], name: 'moveBlock' },
  ],
}])

export const TECH_ENTITY_4_18_REFORM_TECH = reformPart('tech.entity-4.18-reform-tech', [{
  name: ROOM,
  solids: [[480, 360, 16, 136]],
  entities: [
    { id: 'tech-4.18-move-block', kind: 'move_block', bounds: [320, 400, 32, 16], direction: [1, 0], name: 'moveBlock' },
    { id: 'tech-4.18-spikes', kind: 'spikes', bounds: [352, 400, 3, 16], direction: [1, 0], name: 'spikesRight' },
  ],
}])

export const TECH_ENTITY_4_18_1_REFORM_KICK = reformPart('tech.entity-4.18.1-reform-kick', [{
  name: ROOM,
  solids: [[24, 416, 40, 8], [224, 360, 16, 136]],
  entities: [
    { id: 'tech-4.18.1-move-block', kind: 'move_block', bounds: [64, 400, 32, 16], direction: [1, 0], name: 'moveBlock' },
  ],
}])

export const TECH_ENTITY_4_18_3_CORE_BLOCK_ENTITY_DISPLACEMENT = reformPart(
  'tech.entity-4.18.3-core-block-entity-displacement',
  [{
    name: ROOM,
    entities: [
      { id: 'tech-4.18.3-bounce-block', kind: 'bounce_block', bounds: [704, 440, 64, 16], name: 'bounceBlock' },
      { id: 'tech-4.18.3-spikes', kind: 'spikes', bounds: [768, 440, 3, 16], direction: [1, 0], name: 'spikesRight' },
      { id: 'tech-4.18.3-landing', kind: 'jump_thru', bounds: [704, 456, 64, 8], name: 'jumpThru' },
    ],
  }],
)

export const REFORM_MAP_PARTS = [
  TECH_ENTITY_4_17_MOON_BOOST,
  TECH_ENTITY_4_18_REFORM_TECH,
  TECH_ENTITY_4_18_1_REFORM_KICK,
  TECH_ENTITY_4_18_3_CORE_BLOCK_ENTITY_DISPLACEMENT,
] as const
