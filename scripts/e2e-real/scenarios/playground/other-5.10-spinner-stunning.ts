import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { semanticAssert } from "../../verify.js";
import { TECH_OTHER_5_10_SPINNER_STUNNING } from "../cassette-spinner-parts.js";

export const mapParts = [TECH_OTHER_5_10_SPINNER_STUNNING] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "candidate",
  tags: ["feature:spinner", "external:pause"],
  techniqueIds: ["5.10"],
  mapParts,
  name: "other-5.10-spinner-stunning",
  initial: { pos: [100, 496], speed: [0, 0], on_ground: true },
  inputs: Array.from({ length: 20 }, () => input()),
  verify(states) {
    semanticAssert(
      states.some((state) => state.dead),
      scenario.name,
      "baseline spinner never became collidable; pause-window proof cannot be attempted",
    );
  },
});
