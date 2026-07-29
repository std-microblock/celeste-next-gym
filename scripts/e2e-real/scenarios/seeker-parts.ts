import { defineMapPart } from "../map-parts.js";

const PACKAGE = "CelesteGymPlayground";
const SID = "CelesteGymPlayground/Playground";

export const ENTITY_4_19_SEEKER_BOUNCE_PART = defineMapPart({
  id: "tech.entity-4.19-seeker-bounce",
  dependencies: [],
  package: PACKAGE,
  sid: SID,
  rooms: [
    {
      name: "playground",
      bounds: [0, 0, 960, 544],
      spawn: [160, 496],
      solids: [
        [0, 496, 960, 48],
        [112, 256, 16, 240],
      ],
      entities: [
        {
          id: "tech-4.19-seeker",
          kind: "seeker",
          bounds: [194, 478, 12, 12],
          name: "seeker",
        },
      ],
    },
  ],
});
