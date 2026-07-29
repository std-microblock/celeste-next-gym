import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { field, semanticAssert } from "../../verify.js";
import { TECH_OTHER_5_1_4_BINO_EXTENSIONS } from "../lookout-parts.js";

export const mapParts = [TECH_OTHER_5_1_4_BINO_EXTENSIONS] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: ["feature:lookout", "feature:camera-node", "feature:exit-wipe"],
  techniqueIds: ["5.1.4"],
  mapParts,
  recording: {
    primaryFor: ["5.1.4"],
    startFrame: 0,
    endFrame: 640,
    posterFrame: 519,
  },
  name: "other-5.1.4-bino-extensions",
  initial: { pos: [512, 496], speed: [0, 0], on_ground: true },
  inputs: Array.from({ length: 640 }, (_, frame) =>
    input({
      talk_pressed: frame === 0,
      move_y: frame >= 55 && frame < 520 ? -1 : 0,
      // The summit endpoint remains interactive. Jump is the portable
      // MenuCancel mapping, which is the authentic LookRoutine exit event.
      jump_pressed: frame === 520,
      jump_held: frame === 520,
    }),
  ),
  verify(states) {
    const cameras = states.map((state) =>
      field<readonly number[]>(state, "levelCamera"),
    );
    semanticAssert(
      states.some((state) => (field<number>(state, "lookoutNode") ?? 0) >= 2),
      scenario.name,
      "Lookout did not traverse the independent node chain",
    );
    semanticAssert(
      cameras.some(
        (camera) =>
          Math.hypot((camera?.[0] ?? 0) - 352, (camera?.[1] ?? 0) - 364) > 600,
      ),
      scenario.name,
      "node extension never exceeded the 600px FadeWipe threshold",
    );
    semanticAssert(
      field<number>(states[519], "lookoutNode") === 2 &&
        field<number>(states[519], "lookoutNodePercent") === 1 &&
        field<boolean>(states[519], "lookoutInteracting") === true,
      scenario.name,
      "the terminal summit node was not still interactive immediately before MenuCancel",
    );
    semanticAssert(
      states
        .slice(521, 581)
        .every(
          (state) =>
            (state.state === 11 || state.state === "Dummy") &&
            field<boolean>(state, "lookoutInteracting") === true,
        ),
      scenario.name,
      "the summit FadeWipe did not retain Dummy interaction through its full second",
    );
    semanticAssert(
      field<boolean>(states.at(-1), "lookoutInteracting") === false,
      scenario.name,
      "MenuCancel did not complete the summit Lookout exit",
    );
    semanticAssert(
      states.at(-1)?.state === 0,
      scenario.name,
      "the completed summit wipe did not restore Player.StNormal",
    );
  },
});
