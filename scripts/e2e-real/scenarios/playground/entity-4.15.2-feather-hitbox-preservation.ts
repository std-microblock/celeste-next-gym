import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { field, near, pickCore } from "../../verify.js";
import { TECH_ENTITY_4_15_2_HITBOX_PRESERVATION } from "../common-parts.js";

export const mapParts = [TECH_ENTITY_4_15_2_HITBOX_PRESERVATION] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:star-fly"],
  techniqueIds: ["4.15.2"],
  mapParts,
  name: "entity-4.15.2-feather-hitbox-preservation",
  recording: { primaryFor: ["4.15.2"], startFrame: 0, endFrame: 60 },
  initial: { pos: [320, 120], speed: [0, 0] },
  inputs: Array.from({ length: 60 }, () => input({ move_y: 1 })),
  verify(states) {
    const preserved = states.find(
      (state) =>
        state.state === 0 &&
        near(state.speed[1], -140) &&
        JSON.stringify(field<readonly number[]>(state, "playerCollider")) ===
          JSON.stringify([-3, -9, 6, 6]) &&
        JSON.stringify(field<readonly number[]>(state, "playerHurtbox")) ===
          JSON.stringify([-4, -11, 8, 9]),
    );
    if (!preserved || preserved.dead) {
      throw new Error(
        `entity-4.15.2-feather-hitbox-preservation: missing preserved StarFly hurtbox collider + normal hurtbox after IceBall Bounce: ${JSON.stringify(
          states
            .filter((state) => state.state === 0 || near(state.speed[1], -140))
            .map((state) => ({
              core: pickCore(state),
              collider: field(state, "playerCollider"),
              hurtbox: field(state, "playerHurtbox"),
            })),
        )}`,
      );
    }
  },
});
