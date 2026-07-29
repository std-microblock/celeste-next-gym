import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { AREA_1_TARGET } from "../../targets.js";

export const mapParts = [] as const;

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: "active",
  tags: [],
  techniqueIds: ["2.3"],
  recording: { primaryFor: ["2.3"], startFrame: 0, endFrame: 12 },
  mapParts,
  name: "hyper",
  inputs: Array.from({ length: 12 }, (_, frame) =>
    input({
      move_x: 1,
      move_y: frame <= 4 ? 1 : 0,
      jump_pressed: frame === 4,
      jump_held: frame >= 4 && frame < 10,
      dash_pressed: frame === 0,
    }),
  ),
});
