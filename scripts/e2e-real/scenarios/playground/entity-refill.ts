import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { PLAYGROUND_REFILL } from "../common-parts.js";
import { field, semanticAssert } from "../../verify.js";

export const mapParts = [PLAYGROUND_REFILL] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:refill"],
  techniqueIds: [],
  mapParts,
  name: "entity-refill",
  initial: { pos: [168, 496], speed: [0, 0], dashes: 0, stamina: 5 },
  inputs: Array.from({ length: 20 }, () => input()),
  verify(states) {
    const collected = states[1];
    semanticAssert(
      !!collected &&
        collected.dashes === 1 &&
        Math.abs(collected.stamina - 110) <= 0.01,
      "entity-refill",
      "refill did not restore the depleted dash and stamina on the first frame",
    );
    const freeze = states
      .slice(1, 5)
      .map((state) => Number(field(state, "FreezeTimer") ?? 0));
    semanticAssert(
      freeze.some((value) => value > 0.04),
      "entity-refill",
      "the 0.05s RefillRoutine freeze was not observed",
    );
    semanticAssert(
      states.every((state) => !state.dead),
      "entity-refill",
      "player died collecting a refill",
    );
  },
});
