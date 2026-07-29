import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import type { E2EState } from "../../types.js";
import { semanticAssert } from "../../verify.js";
import { ENTITY_4_29_SPRINGBOOST_CANCEL_PART } from "../glider-parts.js";

export const mapParts = [ENTITY_4_29_SPRINGBOOST_CANCEL_PART] as const;
export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "candidate",
  tags: ["feature:glider", "feature:spring"],
  techniqueIds: ["4.29"],
  mapParts,
  name: "entity-4.29-springboost-cancel",
  initial: { pos: [100, 496], on_ground: true },
  inputs: Array.from({ length: 130 }, (_, frame) =>
    input({
      move_x:
        frame >= 14 && frame < 45 ? 1 : frame >= 45 && frame < 75 ? -1 : 0,
      move_y: frame === 35 ? 1 : 0,
      jump_pressed: frame === 25,
      jump_held: frame >= 25 && frame < 34,
      grab_held: frame <= 34 || frame >= 100,
    }),
  ),
  verify: verifySpringboostCancel,
});

function verifySpringboostCancel(states: readonly E2EState[]): void {
  const released = states.findIndex(
    (state, frame) =>
      frame > 0 && !state.holding_glider && states[frame - 1]?.holding_glider,
  );
  const spring = states.findIndex(
    (state, frame) => frame > released && state.speed[1] <= -159.99,
  );
  const regrab = states.findIndex(
    (state, frame) =>
      frame > spring && state.state === 8 && state.holding_glider,
  );
  semanticAssert(
    released > 0 && spring > released,
    scenario.name,
    `release=${released}, spring=${spring}`,
  );
  semanticAssert(regrab > spring, scenario.name, `regrab=${regrab}`);
}
