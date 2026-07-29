import { defineMapPart } from "../map-parts.js";

const PACKAGE = "CelesteGymPlayground";
const SID = "CelesteGymPlayground/Playground";

export const ENTITY_4_28_KORAL_CLIP_PART = defineMapPart({
  id: "tech.entity-4.28-koral-clip",
  dependencies: [],
  package: PACKAGE,
  sid: SID,
  rooms: [
    {
      name: "playground",
      bounds: [0, 0, 960, 544],
      spawn: [540, 496],
      solids: [[0, 520, 960, 24]],
      entities: [
        {
          id: "tech-4.28-gate",
          kind: "temple_gate",
          bounds: [512, 432, 8, 64],
          name: "templeGate",
        },
        {
          id: "tech-4.28-jumpthru",
          kind: "jump_thru",
          bounds: [504, 496, 24, 8],
          name: "jumpThru",
        },
        {
          id: "tech-4.28-theo",
          kind: "theo_crystal",
          bounds: [512, 476, 8, 10],
          name: "theoCrystal",
        },
      ],
    },
  ],
});
