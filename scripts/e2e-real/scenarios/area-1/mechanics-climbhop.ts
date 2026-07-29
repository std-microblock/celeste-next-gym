import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { AREA_1_TARGET } from "../../targets.js";

export const mapParts = [] as const;

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: "active",
  tags: [],
  techniqueIds: ["1.2"],
  recording: { primaryFor: ["1.2"], startFrame: 0, endFrame: 90 },
  mapParts,
  name: "mechanics-climbhop",
  initial: { pos: [140, 112], speed: [0, 30] },
  inputs: Array.from({ length: 90 }, () =>
    input({
      move_x: 1,
      move_y: -1,
      grab_held: true,
    }),
  ),
});
