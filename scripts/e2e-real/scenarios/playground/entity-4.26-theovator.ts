import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { near, semanticAssert } from "../../verify.js";
import { ENTITY_4_26_THEOVATOR_PART } from "../entity-tail-parts.js";

export const mapParts = [ENTITY_4_26_THEOVATOR_PART] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "candidate",
  tags: [],
  techniqueIds: ["4.26"],
  mapParts,
  name: "entity-4.26-theovator",
  initial: { pos: [60, 496], on_ground: true },
  inputs: Array.from({ length: 60 }, (_, frame) =>
    input({
      move_y: frame === 23 ? 1 : frame >= 30 ? -1 : 0,
      dash_pressed: frame === 30,
      grab_held: frame <= 22 || frame >= 36,
    }),
  ),
  verify(states) {
    const pickup = states.find(
      (state, frame) => frame > 30 && state.state === 8 && state.holding_theo,
    );
    const restored = states.find(
      (state, frame) =>
        frame > 30 && states[frame - 1]?.state === 8 && state.state === 0,
    );
    semanticAssert(
      Boolean(pickup),
      scenario.name,
      "updash did not regrab Theo into Pickup",
    );
    semanticAssert(
      Boolean(restored) && near(restored?.speed[1], -240),
      scenario.name,
      `restored=${JSON.stringify(restored)}`,
    );
  },
});
