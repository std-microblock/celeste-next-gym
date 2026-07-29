import { defineMapPart } from "../map-parts.js";

const PACKAGE = "CelesteGymPlayground";
const SID = "CelesteGymPlayground/Playground";
const ROOM = "playground";

export const ELEVEN_JUMP_PART = defineMapPart({
  id: "tech.3.7.7.eleven-jump",
  dependencies: [],
  package: PACKAGE,
  sid: SID,
  rooms: [
    { name: ROOM, bounds: [0, 0, 320, 184], spawn: [24, 160] },
    {
      name: "transition_right",
      bounds: [320, 0, 320, 184],
      spawn: [344, 160],
      solids: [
        [328, 80, 8, 16],
        [416, 128, 80, 8],
      ],
    },
  ],
});

export const REVERSE_CORNERBOOST_PART = defineMapPart({
  id: "tech.3.7.8.reverse-cornerboost",
  dependencies: [],
  package: PACKAGE,
  sid: SID,
  rooms: [
    {
      name: ROOM,
      bounds: [0, 0, 320, 184],
      spawn: [24, 160],
      solids: [[104, 120, 8, 64]],
    },
  ],
});

export const NEUTRAL_REVERSE_CORNERBOOST_PART = defineMapPart({
  id: "tech.3.7.9.neutral-reverse-cornerboost",
  dependencies: [],
  package: PACKAGE,
  sid: SID,
  rooms: [
    {
      name: ROOM,
      bounds: [0, 0, 320, 184],
      spawn: [24, 160],
      solids: [[104, 120, 8, 64]],
    },
  ],
});

export const SPIKED_CORNERBOOST_PART = defineMapPart({
  id: "tech.3.7.10.spiked-cornerboost",
  dependencies: [],
  package: PACKAGE,
  sid: SID,
  rooms: [
    {
      name: ROOM,
      bounds: [0, 0, 320, 184],
      spawn: [24, 160],
      solids: [[40, 40, 8, 64]],
      entities: [
        {
          id: "tech-3-7-10-spikes",
          kind: "spikes",
          bounds: [36, 37, 12, 3],
          direction: [0, -1],
          name: "spikesUp",
        },
      ],
    },
  ],
});
