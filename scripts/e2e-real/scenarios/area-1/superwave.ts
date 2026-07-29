import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { AREA_1_TARGET } from "../../targets.js";

export const mapParts = [] as const;

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: "active",
  tags: [],
  techniqueIds: [],
  mapParts,
  name: "superwave",
  inputs: Array.from({ length: 30 }, (_, frame) =>
    input({
      move_x: frame <= 10 ? 1 : -1,
      move_y: frame >= 11 ? 1 : 0,
      jump_pressed: frame === 10 || frame === 26,
      jump_held: frame === 10 || frame === 26,
      dash_pressed: frame === 0 || frame === 11,
    }),
  ),
});
