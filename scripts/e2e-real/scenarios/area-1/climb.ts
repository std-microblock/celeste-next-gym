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
  name: "climb",
  initial: { pos: [140, 112], speed: [0, 30] },
  inputs: Array.from({ length: 20 }, () =>
    input({
      move_x: 1,
      move_y: -1,
      grab_held: true,
    }),
  ),
});
