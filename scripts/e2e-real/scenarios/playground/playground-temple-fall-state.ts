import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { PLAYGROUND_MISC } from "../common-parts.js";

export const mapParts = [PLAYGROUND_MISC] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:misc-states"],
  techniqueIds: [],
  mapParts,
  name: "playground-temple-fall-state",
  initial: { pos: [200, 300], speed: [0, 0], state: "TempleFall" },
  inputs: Array.from({ length: 140 }, () => input()),
});
