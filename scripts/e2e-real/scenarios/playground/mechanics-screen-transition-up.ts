import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { PLAYGROUND_TRANSITION } from "../common-parts.js";
import type { E2EState } from "../../types.js";
import { field, near, pickCore, semanticAssert } from "../../verify.js";

export const mapParts = [PLAYGROUND_TRANSITION] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: [],
  techniqueIds: ["1.11"],
  recording: {
    primaryFor: ["1.11"],
    startFrame: 0,
    endFrame: 42,
    posterFrame: 6,
  },
  mapParts,
  name: "mechanics-screen-transition-up",
  initial: { pos: [640, 4], speed: [80, -160], dashes: 0, stamina: 20 },
  inputs: Array.from({ length: 42 }, () => input()),
  verify: verifyUpwardScreenTransition,
});

function verifyUpwardScreenTransition(states: readonly E2EState[]): void {
  const entered = states.findIndex(
    (state, frame) =>
      frame > 0 &&
      near(state.speed[0], 0) &&
      near(state.speed[1], -105) &&
      state.dashes === 0 &&
      near(state.stamina, 20),
  );
  semanticAssert(
    entered > 0,
    "mechanics-screen-transition-up",
    `BeforeUpTransition did not apply 0/-105 with delayed resource refill: ${JSON.stringify(states.slice(0, 6).map(pickCore))}`,
  );
  const completed = states.findIndex(
    (state, frame) =>
      frame > entered && state.dashes >= 1 && near(state.stamina, 110),
  );
  semanticAssert(
    completed - entered === 40,
    "mechanics-screen-transition-up",
    `0.65 second transition plus the final coroutine resume took ${completed - entered} frames instead of 40`,
  );
  semanticAssert(
    completed > 0 && near(states[completed]?.pos[1], -5),
    "mechanics-screen-transition-up",
    `upward transition ended at y=${states[completed]?.pos[1]} instead of the source-derived target y=-5`,
  );
}
