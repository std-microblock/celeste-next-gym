import { defineMapPart } from '../map-parts.js'
import type { MapPart } from '../types.js'

const PACKAGE = 'CelesteGymPlayground'
const SID = 'CelesteGymPlayground/Playground'

export const TECH_OTHER_5_2_BUBSDROP: MapPart = defineMapPart({
  id: 'tech.other-5.2-bubsdrop', dependencies: ['playground.base'], package: PACKAGE, sid: SID,
  rooms: [
    { name: 'playground', additionalSpawns: [[440, 496]] },
    {
      name: 'bubsdrop_upper', bounds: [0, -544, 960, 544], spawn: [312, -16],
      solids: [[456, -544, 8, 544]],
      entities: [
        // The unchanged transition auto-jump lands here. A wall jump from the
        // adjacent wall moves left far enough to miss it and fall back below 0.
        { id: 'tech-5.2-jumpthru', kind: 'jump_thru', bounds: [448, -24, 40, 8], name: 'jumpThru' },
      ],
    },
  ],
})

export const BUBS_MAP_PARTS = [TECH_OTHER_5_2_BUBSDROP] as const
