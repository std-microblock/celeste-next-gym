import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { PLAYGROUND_BASE } from "../common-parts.js";

export const mapParts = [PLAYGROUND_BASE] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: [],
  techniqueIds: [],
  mapParts,
  name: "playground-load",
  initial: { pos: [64, 496], speed: [0, 0] },
  inputs: Array.from({ length: 30 }, () => input({ move_x: 1 })),
});
