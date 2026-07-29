import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { PLAYGROUND_WIND } from "../common-parts.js";

export const mapParts = [PLAYGROUND_WIND] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:wind"],
  techniqueIds: [],
  mapParts,
  name: "playground-wind-ground-standing",
  initial: { pos: [820, 248], speed: [0, 0] },
  inputs: Array.from({ length: 45 }, () => input()),
});
