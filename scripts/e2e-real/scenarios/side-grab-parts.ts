import { defineMapPart } from "../map-parts.js";

export const MOVE_BLOCK_SIDE_GRAB_PART = defineMapPart({
  id: "mechanics.move-block-side-grab",
  dependencies: ["playground.base"],
  package: "CelesteGymPlayground",
  sid: "CelesteGymPlayground/Playground",
  rooms: [
    {
      name: "playground",
      entities: [
        {
          id: "mechanics-move-block-side-grab",
          kind: "move_block",
          bounds: [600, 400, 32, 16],
          direction: [-1, 0],
          name: "moveBlock",
        },
      ],
    },
  ],
});
