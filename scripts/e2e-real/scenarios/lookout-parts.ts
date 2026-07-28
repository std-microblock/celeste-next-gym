import { defineMapPart } from '../map-parts.js'
import type { MapPart } from '../types.js'

const PACKAGE = 'CelesteGymPlayground'
const SID = 'CelesteGymPlayground/Playground'

function part(id: string, rooms: MapPart['rooms']): MapPart {
  return defineMapPart({ id, dependencies: ['playground.base'], package: PACKAGE, sid: SID, rooms })
}

export const TECH_OTHER_5_1_BINO_TECH = part('tech.other-5.1-bino-tech', [{
  name: 'playground',
  entities: [{ id: 'tech-5.1-lookout', kind: 'lookout', bounds: [510, 493, 4, 4], name: 'lookout' }],
}])

export const TECH_OTHER_5_1_1_BINO_CLIP = part('tech.other-5.1.1-bino-clip', [{
  name: 'playground',
  entities: [
    { id: 'tech-5.1.1-lookout', kind: 'lookout', bounds: [510, 493, 4, 4], name: 'lookout' },
    { id: 'tech-5.1.1-spinner', kind: 'crystal_static_spinner', bounds: [628, 484, 16, 12], name: 'spinner' },
  ],
}])

export const TECH_OTHER_5_1_2_BINO_CONTROL_STORAGE = part('tech.other-5.1.2-bino-control-storage', [{
  name: 'playground',
  entities: [
    { id: 'tech-5.1.2-lookout', kind: 'lookout', bounds: [510, 493, 4, 4], name: 'lookout' },
    // The vanilla PlayerCollider interrupts StDummy after Talk starts. This
    // is deliberately co-located so the input trace cannot substitute an
    // externally forced state for the real Booster callback.
    { id: 'tech-5.1.2-interrupting-booster', kind: 'booster', bounds: [510, 489, 20, 20], name: 'booster' },
  ],
}])

export const TECH_OTHER_5_1_3_BINO_INTERACTION_STORAGE = part('tech.other-5.1.3-bino-interaction-storage', [
  {
    name: 'playground',
    // Lookout's center is x=958, so the 8px-wide Player hitbox crosses the
    // 960px edge during DummyWalkToExact.  The 4x4 entity itself remains
    // contained by this room, as required by the fixture validator.
    entities: [{ id: 'tech-5.1.3-lookout', kind: 'lookout', bounds: [956, 493, 4, 4], name: 'lookout' }],
  },
  {
    name: 'transition_5_1_3',
    bounds: [960, 0, 960, 544],
    spawn: [984, 496],
    solids: [[960, 496, 960, 48], [1896, 0, 24, 496]],
  },
])

export const TECH_OTHER_5_1_4_BINO_EXTENSIONS = part('tech.other-5.1.4-bino-extensions', [{
  name: 'playground',
  entities: [{
    id: 'tech-5.1.4-lookout',
    kind: 'lookout',
    bounds: [510, 493, 4, 4],
    direction: [0, 1],
    nodes: [[896, 400], [896, 72], [24, 24]],
    name: 'lookout',
  }],
}])

export const LOOKOUT_MAP_PARTS = [
  TECH_OTHER_5_1_BINO_TECH,
  TECH_OTHER_5_1_1_BINO_CLIP,
  TECH_OTHER_5_1_2_BINO_CONTROL_STORAGE,
  TECH_OTHER_5_1_3_BINO_INTERACTION_STORAGE,
  TECH_OTHER_5_1_4_BINO_EXTENSIONS,
] as const
