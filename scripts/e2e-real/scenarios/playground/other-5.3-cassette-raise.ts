import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { semanticAssert } from "../../verify.js";
import { cassetteBlock } from "../cassette-observation.js";
import { TECH_OTHER_5_3_CASSETTE_RAISE } from "../cassette-spinner-parts.js";

export const mapParts = [TECH_OTHER_5_3_CASSETTE_RAISE] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:cassette-block"],
  techniqueIds: ["5.3"],
  mapParts,
  name: "other-5.3-cassette-raise",
  initial: { pos: [96, 496], speed: [0, 0], on_ground: true },
  inputs: Array.from({ length: 100 }, () => input()),
  verify(states) {
    const warned = states.findIndex((state) => {
      const block = cassetteBlock(state, 0);
      return block?.position[1] === 494 && !block.collidable;
    });
    const raised = states.findIndex((state, frame) => {
      const block = cassetteBlock(state, 0);
      return frame > warned && block?.position[1] === 493 && block.collidable;
    });
    semanticAssert(
      warned > 0 && raised > warned,
      scenario.name,
      `collector did not observe cassette WillToggle then collision activation: warned=${warned}, raised=${raised}`,
    );
  },
});
