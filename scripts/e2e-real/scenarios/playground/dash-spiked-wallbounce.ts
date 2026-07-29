import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { PLAYGROUND_SPIKES } from "../common-parts.js";
import type { E2EState } from "../../types.js";
import { field, near, pickCore, semanticAssert } from "../../verify.js";

export const mapParts = [PLAYGROUND_SPIKES] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: [],
  techniqueIds: ["2.10.1"],
  recording: { primaryFor: ["2.10.1"], startFrame: 0, endFrame: 14 },
  mapParts,
  name: "dash-spiked-wallbounce",
  initial: { pos: [396, 207], speed: [0, 0] },
  inputs: Array.from({ length: 14 }, (_, frame) =>
    input({
      move_y: -1,
      jump_pressed: frame === 5,
      jump_held: frame >= 5 && frame < 12,
      dash_pressed: frame === 0,
    }),
  ),
  verify: verifySpikedWallbounce,
});

function verifySpikedWallbounce(states: readonly E2EState[]): void {
  const launch = states[6];
  semanticAssert(
    states.every((state) => !state.dead),
    "dash-spiked-wallbounce",
    "on-time wallbounce touched the directional spikes lethally",
  );
  semanticAssert(
    launch?.state === 0 &&
      near(launch.speed[0], -170) &&
      near(launch.speed[1], -160),
    "dash-spiked-wallbounce",
    `entry-frame launch was ${JSON.stringify(launch?.speed)}`,
  );
}
