import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { AREA_2_TARGET } from "../../targets.js";

export const mapParts = [] as const;

export const scenario = defineScenario({
  target: AREA_2_TARGET,
  room: "1",
  status: "active",
  tags: [],
  techniqueIds: ["4.10"],
  recording: { primaryFor: ["4.10"], startFrame: 0, endFrame: 32 },
  mapParts,
  name: "entity-4.10-dream-jump",
  initial: { pos: [776, -50], speed: [0, 0] },
  inputs: Array.from({ length: 32 }, (_, frame) =>
    input({
      move_x: 1,
      jump_pressed: frame === 15,
      jump_held: frame >= 15 && frame < 25,
      dash_pressed: frame === 0,
    }),
  ),
});
