import { input, inputFrames } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { field, near, semanticAssert } from "../../verify.js";
import { TECH_ENTITY_4_16_LAVA_NEUTRAL } from "../core-heart-squish-parts.js";

export const mapParts = [TECH_ENTITY_4_16_LAVA_NEUTRAL] as const;

const NEUTRAL_FRAME = 169;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "candidate",
  tags: ["feature:rising-lava", "feature:player-collider"],
  techniqueIds: ["4.16"],
  mapParts,
  name: "entity-4.16-lava-neutral",
  initial: {
    pos: [716, 494],
    speed: [0, 0],
    state: 1,
    facing: "Left",
    stamina: 110,
  },
  inputs: inputFrames(220, (frame) =>
    input({
      jump_pressed: frame === NEUTRAL_FRAME,
      jump_held: frame >= NEUTRAL_FRAME && frame < NEUTRAL_FRAME + 8,
      grab_held: frame <= NEUTRAL_FRAME,
    }),
  ),
  verify(states) {
    const neutral = states[NEUTRAL_FRAME + 1];
    semanticAssert(
      neutral != null,
      scenario.name,
      "neutral launch state was not recorded",
    );
    semanticAssert(
      (neutral.state === 0 || neutral.state === "Normal") &&
        near(neutral.speed[1], -105),
      scenario.name,
      `neutral launch=${JSON.stringify(neutral)}`,
    );
    semanticAssert(
      Number(field(neutral, "wallBoostTimer")) > 0,
      scenario.name,
      `wallBoostTimer=${String(field(neutral, "wallBoostTimer"))}`,
    );
    semanticAssert(
      states.slice(0, NEUTRAL_FRAME + 2).every((state) => !state.dead),
      scenario.name,
      "RisingLava killed the player before the one-pixel safe-lip neutral",
    );
  },
});
