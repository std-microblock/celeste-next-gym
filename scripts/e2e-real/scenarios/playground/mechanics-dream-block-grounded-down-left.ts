import { inputFrames } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { TECH_ENTITY_4_10_3_1_DREAM_GRAB_HYPER } from "../common-parts.js";

export const mapParts = [TECH_ENTITY_4_10_3_1_DREAM_GRAB_HYPER] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:dream-block"],
  techniqueIds: [],
  mapParts,
  name: "mechanics-dream-block-grounded-down-left",
  initial: { pos: [250, 432], speed: [0, 0], can_dream_dash: true },
  inputs: inputFrames(20, (frame) => ({
    move_x: -1,
    move_y: 1,
    jump_pressed: false,
    jump_held: false,
    dash_pressed: frame === 0,
    crouch_dash_pressed: false,
    grab_held: false,
    talk_pressed: false,
  })),
  verify(states, context) {
    const entered = states.find((state) => state.state === 9);
    context.assert(
      entered
        && context.near(entered.speed[0], -169.70563)
        && context.near(entered.speed[1], 169.70563),
      "grounded down-left dash did not enter the DreamBlock",
      entered,
    );
  },
});
