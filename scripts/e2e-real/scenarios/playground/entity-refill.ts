import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { PLAYGROUND_REFILL } from "../common-parts.js";
import { semanticAssert } from "../../verify.js";

export const mapParts = [PLAYGROUND_REFILL] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:refill"],
  techniqueIds: [],
  mapParts,
  name: "entity-refill",
  // The player spawns airborne overlapping the refill so the ground dash
  // refill cannot mask the pickup. Frame 1 collects the diamond and restores
  // the depleted dash and stamina while the player keeps falling.
  initial: { pos: [168, 465], speed: [0, 0], dashes: 0, stamina: 5 },
  inputs: Array.from({ length: 40 }, () => input()),
  verify(states) {
    const collected = states[1];
    semanticAssert(
      !!collected &&
        collected.dashes === 1 &&
        Math.abs(collected.stamina - 110) <= 0.01,
      "entity-refill",
      "refill did not restore the depleted dash and stamina on the first frame",
    );
    semanticAssert(
      states.every((state) => !state.dead),
      "entity-refill",
      "player died collecting a refill",
    );
  },
});
