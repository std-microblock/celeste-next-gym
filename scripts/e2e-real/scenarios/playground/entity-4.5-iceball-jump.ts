import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { TECH_ENTITY_4_5_BOUNCE_JUMP } from "../common-parts.js";

export const mapParts = [TECH_ENTITY_4_5_BOUNCE_JUMP] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:booster"],
  techniqueIds: ["4.5"],
  mapParts,
  name: "entity-4.5-iceball-jump",
  recording: { primaryFor: ["4.5"], startFrame: 0, endFrame: 24 },
  initial: { pos: [317, 155], speed: [0, 0] },
  inputs: Array.from({ length: 24 }, (_, frame) =>
    input({
      move_x: 1,
      move_y: 1,
      jump_held: true,
      dash_pressed: frame === 0,
    }),
  ),
  verify(states) {
    const bounced = states.find(
      (state) => Math.abs(state.speed[1] + 140) <= 0.01,
    );
    if (
      states.some((state) => state.dead) ||
      !bounced ||
      bounced.state !== 0 ||
      bounced.speed[0] <= 160 ||
      bounced.dashes !== 1 ||
      Math.abs(bounced.stamina - 110) > 0.01
    ) {
      throw new Error(
        `entity-4.5-iceball-jump: top bounce was ${JSON.stringify(bounced)}`,
      );
    }
  },
});
