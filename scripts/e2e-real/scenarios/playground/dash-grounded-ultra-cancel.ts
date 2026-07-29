import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import type { E2EState } from "../../types.js";
import { near, semanticAssert } from "../../verify.js";
import { GROUNDED_ULTRA_CANCEL_PART } from "../grounded-ultra-cancel-part.js";

export const mapParts = [GROUNDED_ULTRA_CANCEL_PART] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: [],
  techniqueIds: ["2.8.2.1"],
  mapParts,
  name: "dash-grounded-ultra-cancel",
  recording: { primaryFor: ["2.8.2.1"], startFrame: 0, endFrame: 24 },
  initial: { pos: [32, 160], speed: [300, 0], on_ground: true },
  inputs: Array.from({ length: 24 }, (_, frame) =>
    input({
      move_x: 1,
      move_y: 1,
      dash_pressed: frame === 0,
      grab_held: frame >= 5 && frame <= 20,
    }),
  ),
  verify: verifyGroundedUltraCancel,
});

function verifyGroundedUltraCancel(states: readonly E2EState[]): void {
  const pickup = states.findIndex((state) => state.state === 8);
  semanticAssert(
    pickup > 0,
    "dash-grounded-ultra-cancel",
    "Theo pickup did not interrupt Dash",
  );
  const before = states[pickup - 1];
  const lifted = states[pickup];
  semanticAssert(
    before?.state === 2 && near(before.speed[0], 360) && before.ducking,
    "dash-grounded-ultra-cancel",
    `pickup was not preceded by the 360 grounded Ultra: ${JSON.stringify(before)}`,
  );
  semanticAssert(
    lifted &&
      lifted.holding_theo === true &&
      near(lifted.speed[0], 0) &&
      !lifted.ducking,
    "dash-grounded-ultra-cancel",
    `pickup did not unduck and begin its zero-speed tween: ${JSON.stringify(lifted)}`,
  );
  const restored = states.slice(pickup + 1).find((state) => state.state === 0);
  semanticAssert(
    restored && near(restored.speed[0], 360),
    "dash-grounded-ultra-cancel",
    `PickupCoroutine did not restore the pre-DashEnd 360 speed: ${JSON.stringify(restored)}`,
  );
}
