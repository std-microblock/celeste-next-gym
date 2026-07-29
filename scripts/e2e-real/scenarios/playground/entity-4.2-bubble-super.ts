import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { PLAYGROUND_BOOSTER, PLAYGROUND_JUMP_THRU } from "../common-parts.js";
import { field, near, pickCore } from "../../verify.js";

export const mapParts = [PLAYGROUND_BOOSTER, PLAYGROUND_JUMP_THRU] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:booster"],
  techniqueIds: ["4.2"],
  recording: { primaryFor: ["4.2"], startFrame: 0, endFrame: 10 },
  mapParts,
  name: "entity-4.2-bubble-super",
  initial: { pos: [220, 400], speed: [90, 0] },
  inputs: Array.from({ length: 10 }, (_, frame) =>
    input({
      move_x: 1,
      dash_pressed: frame === 5,
      jump_pressed: frame === 9,
      jump_held: frame >= 9,
    }),
  ),
  verify(states) {
    const last = states.at(-1);
    if (
      last?.state !== 0 ||
      Math.abs(last.speed[0] - 260) > 0.01 ||
      Math.abs(last.speed[1] + 105) > 0.01 ||
      last.dashes !== 1
    ) {
      throw new Error(
        "entity-4.2-bubble-super: did not end on the expected 260/-105 super with the booster dash retained",
      );
    }
  },
});
