import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { near, semanticAssert } from "../../verify.js";
import { cassetteBlocks } from "../cassette-observation.js";
import { TECH_OTHER_5_9_TRANSITION_CASSETTE_OFFSET } from "../cassette-spinner-parts.js";

export const mapParts = [TECH_OTHER_5_9_TRANSITION_CASSETTE_OFFSET] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:cassette-block", "feature:transition"],
  techniqueIds: ["5.9"],
  mapParts,
  name: "other-5.9-screen-transition-cassette-offset",
  initial: { pos: [640, 4], speed: [80, -160], dashes: 0, stamina: 20 },
  inputs: Array.from({ length: 90 }, () => input()),
  verify(states) {
    const entered = states.findIndex(
      (state) => near(state.speed[0], 0) && near(state.speed[1], -105),
    );
    const completed = states.findIndex(
      (state, frame) =>
        frame > entered && state.dashes >= 1 && near(state.stamina, 110),
    );
    const sharedPhase = states.findIndex((state, frame) => {
      const blocks = cassetteBlocks(state);
      return (
        frame > entered &&
        blocks.length === 4 &&
        blocks.some(
          (block) =>
            block.index === 0 && block.position[1] === 18 && !block.collidable,
        ) &&
        blocks.some(
          (block) =>
            block.index === 0 && block.position[1] === -46 && !block.collidable,
        ) &&
        blocks.some(
          (block) =>
            block.index === 1 && block.position[1] === 16 && block.collidable,
        ) &&
        blocks.some(
          (block) =>
            block.index === 1 && block.position[1] === -48 && block.collidable,
        )
      );
    });
    semanticAssert(
      entered > 0 && completed > entered,
      scenario.name,
      `transition did not complete around the cassette beat: entered=${entered}, completed=${completed}`,
    );
    semanticAssert(
      sharedPhase > entered && sharedPhase < completed,
      scenario.name,
      `transition did not overlap old/new cassette phases: entered=${entered}, shared=${sharedPhase}, completed=${completed}`,
    );
  },
});
