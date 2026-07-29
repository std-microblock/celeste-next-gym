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
  name: "wall-slide",
  initial: { pos: [140, 96], speed: [0, 60] },
  inputs: Array.from({ length: 20 }, () => input({ move_x: 1 })),
});
