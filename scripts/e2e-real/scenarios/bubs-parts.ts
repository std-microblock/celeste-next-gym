import { defineMapPart } from '../map-parts.js'
import type { MapPart } from '../types.js'

const PACKAGE = 'CelesteGymPlayground'
const SID = 'CelesteGymPlayground/Playground'

export const TECH_OTHER_5_2_BUBSDROP: MapPart = defineMapPart({
  id: 'tech.other-5.2-bubsdrop', dependencies: ['playground.base'], package: PACKAGE, sid: SID,
  rooms: [
    { name: 'playground', solids: [[456, 0, 8, 496]], entities: [
      { id: 'tech-5.2-jumpthru', kind: 'jump_thru', bounds: [488, 432, 112, 8], name: 'jumpThru' },
    ] },
    { name: 'bubsdrop_upper', bounds: [0, -544, 960, 544], spawn: [312, -16], solids: [[456, -544, 8, 496]] },
  ],
})

export const BUBS_MAP_PARTS = [TECH_OTHER_5_2_BUBSDROP] as const
