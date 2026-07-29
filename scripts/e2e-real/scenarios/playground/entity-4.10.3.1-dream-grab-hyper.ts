import { inputFrames } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { TECH_ENTITY_4_10_3_1_DREAM_GRAB_HYPER } from "../common-parts.js";

export const mapParts = [TECH_ENTITY_4_10_3_1_DREAM_GRAB_HYPER] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:dream-block"],
  techniqueIds: ["4.10.3.1"],
  mapParts,
  name: "entity-4.10.3.1-dream-grab-hyper",
  recording: { primaryFor: ["4.10.3.1"], startFrame: 0, endFrame: 75 },
  initial: { pos: [208, 496], speed: [0, 0], can_dream_dash: true },
  inputs: inputFrames(75, (frame) => ({
    move_x: frame >= 28 && frame <= 36 ? -1 : 1,
    move_y: 0,
    jump_pressed: frame === 45,
    jump_held: frame === 45,
    dash_pressed: frame === 0,
    crouch_dash_pressed: frame === 37,
    grab_held: frame >= 28 && frame <= 36,
  })),
  verify(states) {
    const exit = states.findIndex(
      (state, frame) =>
        frame > 0 && state.state === 1 && states[frame - 1]?.state === 9,
    );
    const hyper = states.find(
      (state, frame) =>
        frame > exit &&
        state.state === 0 &&
        Math.abs(state.speed[0] - 325) <= 0.01,
    );
    if (states.some((state) => state.dead) || exit < 0 || !hyper) {
      throw new Error(
        `entity-4.10.3.1-dream-grab-hyper: grab=${exit}, hyper=${JSON.stringify(hyper)}`,
      );
    }
  },
});
