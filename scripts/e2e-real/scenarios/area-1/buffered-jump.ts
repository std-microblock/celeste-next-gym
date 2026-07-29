import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { AREA_1_TARGET } from "../../targets.js";

export const mapParts = [] as const;

export const scenario = defineScenario({
  target: AREA_1_TARGET,
  status: "active",
  tags: [],
  techniqueIds: ["1.9"],
  recording: { primaryFor: ["1.9"], startFrame: 0, endFrame: 14 },
  mapParts,
  name: "buffered-jump",
  initial: { pos: [19, 135], speed: [0, 100] },
  inputs: Array.from({ length: 14 }, (_, frame) =>
    input({
      jump_pressed: frame === 3,
      jump_held: frame >= 3 && frame < 11,
    }),
  ),
});
