import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import type { E2EState } from "../../types.js";
import { near, semanticAssert } from "../../verify.js";
import { ENTITY_4_20_THEO_REGRAB_PART } from "../holdable-parts.js";

export const mapParts = [ENTITY_4_20_THEO_REGRAB_PART] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: [],
  techniqueIds: ["4.20"],
  mapParts,
  name: "entity-4.20-theo-regrab",
  initial: { pos: [60, 496], on_ground: true },
  inputs: Array.from({ length: 60 }, (_, frame) =>
    input({
      move_x: frame >= 14 && frame < 28 ? -1 : frame >= 28 ? 1 : 0,
      move_y: frame === 23 ? 1 : 0,
      dash_pressed: frame === 28,
      grab_held: frame <= 22 || frame >= 35,
    }),
  ),
  verify: verifyTheoRegrab,
});

function verifyTheoRegrab(states: readonly E2EState[]): void {
  const firstPickup = states.findIndex(
    (state) => state.state === 8 && state.holding_theo,
  );
  const released = states.findIndex(
    (state, frame) => frame > firstPickup && !state.holding_theo,
  );
  const regrabbed = states.findIndex(
    (state, frame) =>
      frame > released && state.state === 8 && state.holding_theo,
  );
  const dashBeforeRegrab = regrabbed > 0 ? states[regrabbed - 1] : undefined;
  semanticAssert(
    firstPickup >= 0 && released > firstPickup,
    scenario.name,
    `pickup=${firstPickup}, release=${released}`,
  );
  semanticAssert(
    states
      .slice(released, Math.min(released + 6, states.length))
      .every((state) => !state.holding_theo),
    scenario.name,
    "CannotHold did not block the immediate regrab",
  );
  semanticAssert(
    regrabbed > released &&
      dashBeforeRegrab?.state === 2 &&
      near(dashBeforeRegrab.speed[0], 240),
    scenario.name,
    `regrab=${regrabbed}, before=${JSON.stringify(dashBeforeRegrab)}`,
  );
}
