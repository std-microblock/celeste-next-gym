import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { PLAYGROUND_FALLING_BLOCK_JUMP_OFF } from "../common-parts.js";
import { semanticAssert } from "../../verify.js";

export const mapParts = [PLAYGROUND_FALLING_BLOCK_JUMP_OFF] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:falling-block"],
  techniqueIds: [],
  mapParts,
  name: "entity-falling-block-jump-off",
  // Stand through the 0.2s shake, jump on the shake-end frame, and watch the
  // block drop out from under the airborne player before the 0.4s wait would
  // have elapsed.
  initial: { pos: [216, 400], speed: [0, 0] },
  inputs: Array.from({ length: 40 }, (_, frame) =>
    input({
      jump_pressed: frame === 12,
      jump_held: frame >= 12,
    }),
  ),
  verify(states) {
    const dropped = states.find(
      (state, index) => index > 13 && state.pos[1] > 401,
    );
    semanticAssert(
      !!dropped,
      "entity-falling-block-jump-off",
      "block did not drop early after the player jumped off",
    );
    semanticAssert(
      states.slice(0, 15).every((state) => !state.dead),
      "entity-falling-block-jump-off",
      "player died during the shake window",
    );
  },
});
