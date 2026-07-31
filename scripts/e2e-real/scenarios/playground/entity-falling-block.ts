import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { PLAYGROUND_FALLING_BLOCK } from "../common-parts.js";
import { near, pickCore, semanticAssert } from "../../verify.js";

export const mapParts = [PLAYGROUND_FALLING_BLOCK] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:falling-block"],
  techniqueIds: [],
  mapParts,
  name: "entity-falling-block",
  // The player spawns standing on the 32x16 block (top y=400). The rider
  // check triggers the drop; the block carries the player down and lands on
  // the main floor (y=496), resting with its top at y=480.
  initial: { pos: [216, 400], speed: [0, 0] },
  inputs: Array.from({ length: 90 }, () => input()),
  verify(states) {
    const falling = states.find(
      (state, index) => index > 12 && state.pos[1] > 401,
    );
    semanticAssert(
      !!falling,
      "entity-falling-block",
      "block never carried the player downward",
    );
    const landed = states.at(-1);
    semanticAssert(
      !!landed &&
        landed.on_ground &&
        near(landed.pos[1], 480) &&
        !landed.dead,
      "entity-falling-block",
      "player did not land on the fallen block: " + JSON.stringify(landed && pickCore(landed)),
    );
  },
});
