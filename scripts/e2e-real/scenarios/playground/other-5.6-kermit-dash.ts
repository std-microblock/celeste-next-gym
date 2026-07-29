import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import type { E2EState } from "../../types.js";
import { field, near, pickCore, semanticAssert } from "../../verify.js";
import { PLAYGROUND_OTHER_KERMIT } from "../common-parts.js";

export const mapParts = [PLAYGROUND_OTHER_KERMIT] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: [],
  techniqueIds: ["5.6"],
  recording: { primaryFor: ["5.6"], startFrame: 0, endFrame: 47 },
  mapParts,
  name: "other-5.6-kermit-dash",
  initial: { pos: [630, 12], speed: [0, 0], dashes: 1 },
  inputs: Array.from({ length: 47 }, (_, frame) =>
    input({ move_y: -1, dash_pressed: frame === 0 }),
  ),
  verify: verifyKermitDash,
});

function verifyKermitDash(states: readonly E2EState[]): void {
  const entered = states.findIndex(
    (state, frame) =>
      frame > 0 &&
      (state.state === "Normal" || state.state === 0) &&
      near(state.speed[1], -105),
  );
  semanticAssert(
    entered > 0,
    "other-5.6-kermit-dash",
    `upward transition did not cancel Dash into Normal: ${JSON.stringify(states.slice(0, 6).map(pickCore))}`,
  );
  const dashDir = field<readonly number[]>(states[entered]!, "DashDir");
  const dashAttack = field<number>(states[entered]!, "dashAttackTimer");
  semanticAssert(
    Array.isArray(dashDir) &&
      near(Number(dashDir[0]), 0) &&
      near(Number(dashDir[1]), -1),
    "other-5.6-kermit-dash",
    `transition lost upward dashDir: ${JSON.stringify(dashDir)}`,
  );
  semanticAssert(
    typeof dashAttack === "number" && dashAttack > 0,
    "other-5.6-kermit-dash",
    `transition lost dash attack timer: ${JSON.stringify(dashAttack)}`,
  );
  const completed = states.findIndex(
    (state, frame) =>
      frame > entered &&
      state.dashes === 1 &&
      near(state.stamina, 110) &&
      near(state.pos[1], -5),
  );
  semanticAssert(
    completed > entered,
    "other-5.6-kermit-dash",
    `transition did not complete with resource refills: ${JSON.stringify(states.slice(-8).map(pickCore))}`,
  );
  const completedDir = field<readonly number[]>(states[completed]!, "DashDir");
  const completedAttack = field<number>(states[completed]!, "dashAttackTimer");
  semanticAssert(
    completedDir &&
      near(completedDir[0], 0) &&
      near(completedDir[1], -1) &&
      completedAttack !== undefined &&
      completedAttack > 0,
    "other-5.6-kermit-dash",
    `completion lost Kermit dash direction or attack timer: ${JSON.stringify({ completedDir, completedAttack })}`,
  );
}
