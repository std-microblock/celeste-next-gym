import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import type { E2EState } from "../../types.js";
import { near, semanticAssert } from "../../verify.js";
import { ENTITY_4_22_NEUTRAL_DROP_PART } from "../holdable-parts.js";

export const mapParts = [ENTITY_4_22_NEUTRAL_DROP_PART] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: [],
  techniqueIds: ["4.22"],
  mapParts,
  name: "entity-4.22-neutral-drop",
  recording: { primaryFor: ["4.22"], startFrame: 0, endFrame: 48 },
  initial: { pos: [140, 496], on_ground: true },
  inputs: Array.from({ length: 48 }, (_, frame) =>
    input({
      move_y: frame === 23 ? 1 : 0,
      grab_held: frame <= 22 || frame >= 30,
    }),
  ),
  verify: verifyNeutralDrop,
});

function verifyNeutralDrop(states: readonly E2EState[]): void {
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
  semanticAssert(
    firstPickup >= 0 && released > firstPickup,
    scenario.name,
    `pickup=${firstPickup}, release=${released}`,
  );
  semanticAssert(
    near(states[released]?.speed[0], 0),
    scenario.name,
    `neutral drop applied horizontal recoil: ${JSON.stringify(states[released])}`,
  );
  semanticAssert(
    states
      .slice(released, Math.min(released + 6, states.length))
      .every((state) => !state.holding_theo),
    scenario.name,
    "CannotHold did not remain observable after the zero-force release",
  );
  semanticAssert(
    regrabbed > released,
    scenario.name,
    `Theo did not remain in place for regrab: ${regrabbed}`,
  );
}
