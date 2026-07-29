import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { PLAYGROUND_BOOSTER } from "../common-parts.js";

export const mapParts = [PLAYGROUND_BOOSTER] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:booster"],
  techniqueIds: ["1.7"],
  recording: { primaryFor: ["1.7"], startFrame: 0, endFrame: 24 },
  mapParts,
  name: "playground-green-booster-right",
  initial: { pos: [760, 440], speed: [0, 0] },
  inputs: Array.from({ length: 24 }, (_, frame) =>
    input({
      move_x: 1,
      dash_pressed: frame === 1,
    }),
  ),
});
