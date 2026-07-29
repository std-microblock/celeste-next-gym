import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import type { E2EState } from "../../types.js";
import { field, near, pickCore, semanticAssert } from "../../verify.js";
import { PLAYGROUND_OTHER_SUBPIXEL } from "../common-parts.js";

export const mapParts = [PLAYGROUND_OTHER_SUBPIXEL] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: [],
  techniqueIds: ["5.12"],
  recording: { primaryFor: ["5.12"], startFrame: 0, endFrame: 5 },
  mapParts,
  name: "other-5.12-subpixel-manipulation",
  initial: { pos: [160, 80], speed: [0, 0] },
  inputs: Array.from({ length: 5 }, (_, frame) =>
    input({ move_x: frame % 2 === 0 ? 1 : -1 }),
  ),
  verify: verifySubpixelManipulation,
});

function verifySubpixelManipulation(states: readonly E2EState[]): void {
  const remainder = (frame: number): readonly number[] | undefined =>
    field<readonly number[]>(states[frame]!, "movementCounter");
  semanticAssert(
    states.length === 6,
    "other-5.12-subpixel-manipulation",
    `expected six states: ${JSON.stringify(states.map(pickCore))}`,
  );
  semanticAssert(
    near(states[1]?.pos[0] ?? 0, 160) &&
      near(remainder(1)?.[0] ?? 0, 0.180556, 0.00001),
    "other-5.12-subpixel-manipulation",
    `first input did not remain subpixel-only: ${JSON.stringify(remainder(1))}`,
  );
  semanticAssert(
    near(states[3]?.pos[0] ?? 0, 160) &&
      near(remainder(3)?.[0] ?? 0, 0.361113, 0.00001),
    "other-5.12-subpixel-manipulation",
    `second positive pulse did not accumulate remainder: ${JSON.stringify(remainder(3))}`,
  );
  semanticAssert(
    near(states[5]?.pos[0] ?? 0, 161) &&
      near(remainder(5)?.[0] ?? 0, -0.458331, 0.00001),
    "other-5.12-subpixel-manipulation",
    `half-pixel crossing did not move one pixel and retain the signed remainder: ${JSON.stringify(remainder(5))}`,
  );
  semanticAssert(
    !states[5]?.on_ground && !states[5]?.ducking && !states[5]?.dead,
    "other-5.12-subpixel-manipulation",
    "subpixel pulses changed unrelated core state",
  );
}
