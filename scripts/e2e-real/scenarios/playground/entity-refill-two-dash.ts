import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { PLAYGROUND_REFILL_TWO_DASH } from "../common-parts.js";
import { semanticAssert } from "../../verify.js";

export const mapParts = [PLAYGROUND_REFILL_TWO_DASH] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:refill"],
  techniqueIds: [],
  mapParts,
  name: "entity-refill-two-dash",
  initial: { pos: [168, 496], speed: [0, 0], dashes: 0, stamina: 5 },
  inputs: Array.from({ length: 20 }, () => input()),
  verify(states) {
    const collected = states[1];
    semanticAssert(
      !!collected && collected.dashes === 2,
      "entity-refill-two-dash",
      "pink twoDash refill did not grant two dashes",
    );
    semanticAssert(
      states.every((state) => !state.dead),
      "entity-refill-two-dash",
      "player died collecting a pink refill",
    );
  },
});
