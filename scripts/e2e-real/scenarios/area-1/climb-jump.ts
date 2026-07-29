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
  name: "climb-jump",
  initial: { pos: [140, 112], speed: [0, 30] },
  inputs: Array.from({ length: 16 }, (_, frame) =>
    input({
      move_x: 1,
      grab_held: frame <= 8,
      jump_pressed: frame === 8,
      jump_held: frame >= 8 && frame < 14,
    }),
  ),
});
