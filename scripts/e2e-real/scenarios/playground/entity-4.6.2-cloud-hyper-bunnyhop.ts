import { inputFrames } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { TECH_ENTITY_4_6_2_CLOUD_HYPER_BUNNYHOP } from "../common-parts.js";

export const mapParts = [TECH_ENTITY_4_6_2_CLOUD_HYPER_BUNNYHOP] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:cloud"],
  techniqueIds: ["4.6.2"],
  mapParts,
  name: "entity-4.6.2-cloud-hyper-bunnyhop",
  initial: { pos: [520, 434], speed: [0, 0] },
  inputs: inputFrames(45, (frame) => ({
    move_x: frame >= 24 && frame <= 28 ? -1 : frame >= 29 ? 1 : 0,
    move_y: 0,
    jump_pressed: frame === 29 || frame === 38,
    jump_held: frame === 29 || frame === 38,
    dash_pressed: false,
    crouch_dash_pressed: frame === 24,
    grab_held: false,
  })),
  verify(states) {
    const hyper = states.findIndex(
      (state) => state.state === 0 && Math.abs(state.speed[0] - 325) <= 0.01,
    );
    const bunnyhop = states.find(
      (state, frame) =>
        frame > hyper &&
        state.state === 0 &&
        !state.on_ground &&
        state.speed[0] > 250 &&
        state.speed[1] <= -105,
    );
    if (states.some((state) => state.dead) || hyper < 0 || !bunnyhop) {
      throw new Error(
        `entity-4.6.2-cloud-hyper-bunnyhop: hyper=${hyper}, bunnyhop=${JSON.stringify(bunnyhop)}`,
      );
    }
  },
});
