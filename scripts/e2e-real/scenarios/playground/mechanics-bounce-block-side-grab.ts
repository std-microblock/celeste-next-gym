import { input, inputFrames } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { field, near, pickCore, semanticAssert } from "../../verify.js";
import { PLAYGROUND_BOUNCE_BLOCK } from "../common-parts.js";

export const mapParts = [PLAYGROUND_BOUNCE_BLOCK] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:bounce-block"],
  techniqueIds: [],
  mapParts,
  name: "mechanics-bounce-block-side-grab",
  initial: { pos: [420, 368], speed: [0, 0], facing: "Left", stamina: 110 },
  inputs: inputFrames(48, () => input({ grab_held: true })),
  verify(states) {
    const grabbed = states.findIndex(
      (state) => state.state === "Climb" || state.state === 1,
    );
    const carried = states.findIndex(
      (state, frame) =>
        frame > grabbed &&
        (state.state === "Climb" || state.state === 1) &&
        state.pos[0] < 419.99,
    );
    const released = states.findIndex(
      (state, frame) =>
        frame > carried &&
        (state.state === "Normal" || state.state === 0) &&
        near(field<number>(state, "jumpGraceTimer"), 0.1, 0.001),
    );
    semanticAssert(
      grabbed >= 0,
      scenario.name,
      `stationary left-facing player did not grab the BounceBlock: ${JSON.stringify(states.slice(0, 4).map(pickCore))}`,
    );
    semanticAssert(
      carried > grabbed,
      scenario.name,
      `BounceBlock wind-up did not carry its side rider: ${JSON.stringify(states.map(pickCore))}`,
    );
    semanticAssert(
      released > carried,
      scenario.name,
      `BounceBlock did not use the source ShakeOffPlayer release: ${JSON.stringify(states.map(pickCore))}`,
    );
    semanticAssert(
      !states.some((state) => state.dead),
      scenario.name,
      "player died during the BounceBlock side grab",
    );
  },
});
