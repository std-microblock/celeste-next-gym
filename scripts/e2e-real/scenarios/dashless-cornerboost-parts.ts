import { defineMapPart } from '../map-parts.js'

const PACKAGE = 'CelesteGymPlayground'
const SID = 'CelesteGymPlayground/Playground'
const ROOM = 'playground'

export const ELEVEN_JUMP_PART = defineMapPart({
  id: 'tech.3.7.7.eleven-jump',
  dependencies: ['playground.base'],
  package: PACKAGE,
  sid: SID,
  rooms: [{
    name: 'transition_right',
    bounds: [960, 0, 960, 544],
    spawn: [968, 496],
    solids: [
      [968, 120, 8, 16],
      [1056, 168, 80, 8],
    ],
  }],
})

export const REVERSE_CORNERBOOST_PART = defineMapPart({
  id: 'tech.3.7.8.reverse-cornerboost',
  dependencies: ['playground.base'],
  package: PACKAGE,
  sid: SID,
  rooms: [{ name: ROOM, solids: [[120, 240, 8, 64]] }],
})

export const NEUTRAL_REVERSE_CORNERBOOST_PART = defineMapPart({
  id: 'tech.3.7.9.neutral-reverse-cornerboost',
  dependencies: ['playground.base'],
  package: PACKAGE,
  sid: SID,
  rooms: [{ name: ROOM, solids: [[120, 240, 8, 64]] }],
})

export const SPIKED_CORNERBOOST_PART = defineMapPart({
  id: 'tech.3.7.10.spiked-cornerboost',
  dependencies: ['playground.base'],
  package: PACKAGE,
  sid: SID,
  rooms: [{
    name: ROOM,
    solids: [[240, 240, 8, 64]],
    entities: [{
      id: 'tech-3-7-10-spikes',
      kind: 'spikes',
      bounds: [236, 237, 12, 3],
      direction: [0, -1],
      name: 'spikesUp',
    }],
  }],
})
