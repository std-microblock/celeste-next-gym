import { inputFrames } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import type { E2EState } from "../../types.js";
import { field, semanticAssert } from "../../verify.js";
import { TECH_OTHER_5_8_ROBOBOOST } from "../common-parts.js";

export const mapParts = [TECH_OTHER_5_8_ROBOBOOST] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "candidate",
  tags: ["feature:move-block"],
  techniqueIds: ["5.8"],
  mapParts,
  name: "other-5.8-roboboost",
  initial: { pos: [432, 464], speed: [0, 0], on_ground: true },
  inputs: inputFrames(90, (frame) => ({
    move_x: frame >= 45 && frame < 58 ? 1 : frame >= 58 ? -1 : 0,
    move_y: 0,
    jump_pressed: frame === 49 || frame === 51,
    jump_held: frame === 49 || frame === 51,
    dash_pressed: false,
    crouch_dash_pressed: frame === 45,
    grab_held: frame === 51,
  })),
  verify: verifyRoboboostCandidate,
});

function verifyRoboboostCandidate(states: readonly E2EState[]): void {
  const hyper = states.findIndex((state) => state.speed[0] > 300);
  const retained = states.findIndex(
    (state, frame) =>
      frame > hyper &&
      Number(field<number>(state, "wallSpeedRetentionTimer") ?? 0) > 0 &&
      Number(field<number>(state, "wallSpeedRetained") ?? 0) > 300,
  );
  const restored = states.findIndex(
    (state, frame) => frame > retained && state.speed[0] > 300,
  );
  semanticAssert(
    hyper > 0 &&
      retained > hyper &&
      restored > retained &&
      !states.some((state) => state.dead),
    scenario.name,
    `MoveBlock hyper / reverse corner retention chain incomplete: hyper=${hyper}, retained=${retained}, restored=${restored}`,
  );
}
