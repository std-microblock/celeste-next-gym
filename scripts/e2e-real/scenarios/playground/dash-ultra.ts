import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { PLAYGROUND_BASE } from "../common-parts.js";
import { verifyUltra } from "../shared/ultra.js";

export const mapParts = [PLAYGROUND_BASE] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: [],
  techniqueIds: ["2.8"],
  recording: { primaryFor: ["2.8"], startFrame: 0, endFrame: 12 },
  mapParts,
  name: "dash-ultra",
  initial: { pos: [200, 480], speed: [0, 0] },
  inputs: Array.from({ length: 12 }, (_, frame) =>
    input({
      move_x: 1,
      move_y: 1,
      dash_pressed: frame === 0,
    }),
  ),
  verify: verifyUltra,
});
