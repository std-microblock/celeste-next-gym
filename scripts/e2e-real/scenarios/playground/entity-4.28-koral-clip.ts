import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { semanticAssert } from "../../verify.js";
import { field } from "../../verify.js";
import { ENTITY_4_28_KORAL_CLIP_PART } from "../koral-parts.js";

export const mapParts = [ENTITY_4_28_KORAL_CLIP_PART] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: [],
  techniqueIds: ["4.28"],
  mapParts,
  name: "entity-4.28-koral-clip",
  recording: { primaryFor: ["4.28"], startFrame: 0, endFrame: 90 },
  initial: { pos: [540, 496], on_ground: true },
  inputs: Array.from({ length: 90 }, (_, frame) =>
    input({
      move_x: frame >= 8 ? -1 : 0,
      grab_held: frame >= 55,
    }),
  ),
  verify(states) {
    const gateStop = states.findIndex(
      (state, frame) =>
        frame > 8 &&
        state.on_ground &&
        state.pos[0] <= 525 &&
        state.pos[0] >= 523,
    );
    semanticAssert(
      states.every((state) => !state.dead),
      scenario.name,
      "TempleGate close killed the player",
    );
    semanticAssert(
      gateStop >= 0,
      scenario.name,
      "CloseBehindPlayerAlways did not become a solid before the return walk",
    );
    const clippedTheo = states.find((state) => {
      const position = field<readonly number[]>(state, "theoPosition");
      return (
        position?.length === 2 &&
        (Math.abs(Number(position[0]) - 512) > 0.01 ||
          Math.abs(Number(position[1]) - 476) > 0.01)
      );
    });
    semanticAssert(
      clippedTheo,
      scenario.name,
      "TempleGate never displaced Theo from its source position",
    );
  },
});
