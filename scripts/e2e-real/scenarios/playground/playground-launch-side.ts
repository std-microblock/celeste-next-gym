import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { PLAYGROUND_LAUNCH } from "../common-parts.js";

export const mapParts = [PLAYGROUND_LAUNCH] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:launch"],
  techniqueIds: [],
  mapParts,
  name: "playground-launch-side",
  initial: { pos: [500, 300], speed: [280, -150], state: "Launch" },
  inputs: Array.from({ length: 80 }, () => input({ move_x: 1 })),
});
