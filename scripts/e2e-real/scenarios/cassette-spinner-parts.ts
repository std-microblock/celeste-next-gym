import { defineMapPart } from '../map-parts.js'
import type { MapPart } from '../types.js'

const PACKAGE = 'CelesteGymPlayground'
const SID = 'CelesteGymPlayground/Playground'
const ROOM = 'playground'

function part(id: string, rooms: MapPart['rooms']): MapPart {
  return defineMapPart({ id, dependencies: ['playground.base'], package: PACKAGE, sid: SID, rooms })
}

export const TECH_ENTITY_4_18_2_REFORM_BOOST = part('tech.entity-4.18.2-reform-boost-cassette-boost', [{
  name: ROOM,
  entities: [
    { id: 'tech-4.18.2-cassette-0', kind: 'cassette_block', bounds: [64, 493, 128, 16], direction: [0, 1], name: 'cassetteBlock' },
    { id: 'tech-4.18.2-cassette-1', kind: 'cassette_block', bounds: [320, 400, 64, 16], direction: [1, 1], name: 'cassetteBlock' },
  ],
}])

export const TECH_ENTITY_4_18_2_1_CASSOOSTED_FUPER = part('tech.entity-4.18.2.1-cassoosted-fuper', [{
  name: ROOM,
  entities: [
    { id: 'tech-4.18.2.1-feather', kind: 'fly_feather', bounds: [490, 474, 20, 20], name: 'infiniteStar' },
    { id: 'tech-4.18.2.1-cassette-0', kind: 'cassette_block', bounds: [448, 493, 240, 16], direction: [0, 1], name: 'cassetteBlock' },
    { id: 'tech-4.18.2.1-cassette-1', kind: 'cassette_block', bounds: [720, 400, 64, 16], direction: [1, 1], name: 'cassetteBlock' },
  ],
}])

export const TECH_OTHER_5_3_CASSETTE_RAISE = part('tech.other-5.3-cassette-raise', [{
  name: ROOM,
  entities: [
    { id: 'tech-5.3-cassette-0', kind: 'cassette_block', bounds: [64, 493, 128, 16], direction: [0, 1], name: 'cassetteBlock' },
    { id: 'tech-5.3-cassette-1', kind: 'cassette_block', bounds: [320, 400, 64, 16], direction: [1, 1], name: 'cassetteBlock' },
  ],
}])

export const TECH_OTHER_5_9_TRANSITION_CASSETTE_OFFSET = part('tech.other-5.9-screen-transition-cassette-offset', [
  {
    name: ROOM,
    entities: [
      { id: 'tech-5.9-old-cassette-0', kind: 'cassette_block', bounds: [608, 16, 96, 16], direction: [0, 1], name: 'cassetteBlock' },
      { id: 'tech-5.9-old-cassette-1', kind: 'cassette_block', bounds: [736, 16, 96, 16], direction: [1, 1], name: 'cassetteBlock' },
    ],
  },
  {
    name: 'transition_5_9',
    bounds: [0, -544, 960, 544],
    spawn: [640, -16],
    entities: [
      { id: 'tech-5.9-new-cassette-0', kind: 'cassette_block', bounds: [608, -48, 96, 16], direction: [0, 1], name: 'cassetteBlock' },
      { id: 'tech-5.9-new-cassette-1', kind: 'cassette_block', bounds: [736, -48, 96, 16], direction: [1, 1], name: 'cassetteBlock' },
    ],
  },
])

export const TECH_OTHER_5_10_SPINNER_STUNNING = part('tech.other-5.10-spinner-stunning', [{
  name: ROOM,
  entities: [{ id: 'tech-5.10-spinner', kind: 'crystal_static_spinner', bounds: [92, 484, 16, 12], name: 'spinner' }],
}])

export const TECH_OTHER_5_11_SPINNER_FREEZE = part('tech.other-5.11-spinner-freeze', [{
  name: ROOM,
  entities: [{ id: 'tech-5.11-spinner', kind: 'crystal_static_spinner', bounds: [212, 394, 16, 12], name: 'spinner' }],
}])

export const CASSETTE_SPINNER_MAP_PARTS = [
  TECH_ENTITY_4_18_2_REFORM_BOOST,
  TECH_ENTITY_4_18_2_1_CASSOOSTED_FUPER,
  TECH_OTHER_5_3_CASSETTE_RAISE,
  TECH_OTHER_5_9_TRANSITION_CASSETTE_OFFSET,
  TECH_OTHER_5_10_SPINNER_STUNNING,
  TECH_OTHER_5_11_SPINNER_FREEZE,
] as const
