import { inputFrames } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { TECH_ENTITY_4_10_3_DREAM_SMUGGLE } from "../common-parts.js";

export const mapParts = [TECH_ENTITY_4_10_3_DREAM_SMUGGLE] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:dream-block", "feature:theo"],
  techniqueIds: ["4.10.3"],
  mapParts,
  name: "entity-4.10.3-dream-smuggle",
  recording: { primaryFor: ["4.10.3"], startFrame: 0, endFrame: 80 },
  initial: { pos: [208, 496], speed: [0, 0], can_dream_dash: true },
  inputs: inputFrames(80, (frame) => ({
    move_x: 1,
    move_y: 0,
    jump_pressed: false,
    jump_held: false,
    dash_pressed: frame === 0,
    crouch_dash_pressed: false,
    grab_held: true,
  })),
  verify(states) {
    const pickup = states.find(
      (state) => state.state === 8 && state.holding_theo,
    );
    const inside = states.find(
      (state) => state.state === 9 && state.holding_theo,
    );
    const exited = states.find(
      (state, frame) =>
        frame > 0 &&
        state.state !== 9 &&
        states[frame - 1]?.state === 9 &&
        state.holding_theo,
    );
    if (states.some((state) => state.dead) || !pickup || !inside || !exited) {
      throw new Error(
        `entity-4.10.3-dream-smuggle: pickup=${!!pickup}, inside=${!!inside}, exit=${JSON.stringify(exited)}`,
      );
    }
  },
});
