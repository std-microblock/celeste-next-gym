import { input, inputFrames } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { field, semanticAssert } from "../../verify.js";
import { TECH_ENTITY_4_18_REFORM_TECH } from "../reform-parts.js";

export const mapParts = [TECH_ENTITY_4_18_REFORM_TECH] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:move-block"],
  techniqueIds: ["4.18"],
  recording: { primaryFor: ["4.18"], startFrame: 0, endFrame: 370 },
  mapParts,
  name: "entity-4.18-reform-tech",
  initial: { pos: [336, 400], speed: [0, 0] },
  inputs: inputFrames(370, () => input()),
  verify(states) {
    const broken = states.findIndex(
      (state) =>
        field(state, "reformBlockCollidable") === false &&
        field(state, "reformBlockVisible") === false,
    );
    const body = states.findIndex(
      (state, frame) =>
        frame > broken &&
        field(state, "reformBlockCollidable") === true &&
        field(state, "reformBlockVisible") === false,
    );
    const visible = states.findIndex(
      (state, frame) =>
        frame > body &&
        field(state, "reformBlockCollidable") === true &&
        field(state, "reformBlockVisible") === true &&
        field(state, "reformSpikeCollidable") === true,
    );
    const restored = field<readonly number[]>(
      states[body],
      "reformBlockPosition",
    );

    semanticAssert(
      broken >= 0,
      scenario.name,
      "MoveBlock never entered its non-collidable break phase",
    );
    semanticAssert(
      body > broken,
      scenario.name,
      "MoveBlock body did not reform before becoming visible",
    );
    semanticAssert(
      field(states[body], "reformSpikeCollidable") === false,
      scenario.name,
      "attached spikes re-enabled with the invisible body",
    );
    semanticAssert(
      visible > body,
      scenario.name,
      "MoveBlock visuals/static movers never followed the body reform",
    );
    semanticAssert(
      body - broken >= 125 && body - broken <= 140,
      scenario.name,
      `body reform gap was ${body - broken} frames instead of about 2.2 seconds`,
    );
    semanticAssert(
      visible - body >= 45 && visible - body <= 52,
      scenario.name,
      `visibility gap was ${visible - body} frames instead of about 0.8 seconds`,
    );
    semanticAssert(
      restored?.length === 2 &&
        Math.abs(Number(restored[0]) - 320) <= 0.01 &&
        Math.abs(Number(restored[1]) - 400) <= 0.01,
      scenario.name,
      `body restored at ${JSON.stringify(restored)}`,
    );
  },
});
