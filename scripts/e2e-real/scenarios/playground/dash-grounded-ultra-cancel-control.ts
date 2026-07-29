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
  name: "dash-grounded-ultra-cancel-control",
  initial: { pos: [32, 160], speed: [300, 0], on_ground: true },
  inputs: Array.from({ length: 24 }, (_, frame) =>
    input({
      move_x: 1,
      move_y: 1,
      dash_pressed: frame === 0,
    }),
  ),
  verify: verifyGroundedUltraCancelControl,
});

function verifyGroundedUltraCancelControl(states: readonly E2EState[]): void {
  semanticAssert(
    states.every((state) => state.state !== 8 && state.holding_theo !== true),
    "dash-grounded-ultra-cancel-control",
    "control unexpectedly picked up Theo",
  );
  semanticAssert(
    states.some(
      (state) =>
        state.state === 2 && near(state.speed[0], 360) && state.ducking,
    ),
    "dash-grounded-ultra-cancel-control",
    "control never formed the 360 grounded Ultra",
  );
  const normalized = states.find(
    (state) => state.state === 0 && near(state.speed[0], 160),
  );
  semanticAssert(
    normalized,
    "dash-grounded-ultra-cancel-control",
    `natural DashCoroutine never normalized 360 to 160: ${JSON.stringify(states.map((state) => state.speed[0]))}`,
  );
}
