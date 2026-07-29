import { defineMapPart } from "../map-parts.js";

export const GROUNDED_ULTRA_CANCEL_PART = defineMapPart({
  id: "tech.2.8.2.1.grounded-ultra-cancel",
  dependencies: [],
  package: "CelesteGymPlayground",
  sid: "CelesteGymPlayground/Playground",
  rooms: [
    {
      name: "playground",
      bounds: [0, 0, 320, 184],
      spawn: [24, 160],
      solids: [[0, 160, 320, 24]],
      entities: [
        {
          id: "tech-2-8-2-1-theo",
          kind: "theo_crystal",
          bounds: [64, 150, 8, 10],
          name: "theoCrystal",
        },
      ],
    },
  ],
});
