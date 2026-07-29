import { input } from "../../inputs.js";
import { defineScenario } from "../../scenario.js";
import { PLAYGROUND_TARGET } from "../../targets.js";
import { PLAYGROUND_SPRING } from "../common-parts.js";
import type { E2EState } from "../../types.js";
import { field, near, pickCore, semanticAssert } from "../../verify.js";

export const mapParts = [PLAYGROUND_SPRING] as const;

export const scenario = defineScenario({
  target: PLAYGROUND_TARGET,
  status: "active",
  tags: [],
  techniqueIds: ["2.1"],
  recording: { primaryFor: ["2.1"], startFrame: 0, endFrame: 16 },
  mapParts,
  name: "dash-spring-cancel",
  initial: { pos: [80, 488], speed: [0, 100], dashes: 0 },
  inputs: Array.from({ length: 16 }, (_, frame) =>
    input({
      dash_pressed: frame === 0,
    }),
  ),
  verify: verifySpringCancel,
});

function verifySpringCancel(states: readonly E2EState[]): void {
  const dash = states.find((state) => state.state === 2);
  semanticAssert(dash, "dash-spring-cancel", "Dash state was not observed");
  const beforeDashSpeed = field(dash, "beforeDashSpeed");
  semanticAssert(
    Array.isArray(beforeDashSpeed),
    "dash-spring-cancel",
    "Dash did not expose beforeDashSpeed",
  );
  semanticAssert(
    near(beforeDashSpeed[0], 0) && near(beforeDashSpeed[1], -185),
    "dash-spring-cancel",
    `Dash replaced ${JSON.stringify(beforeDashSpeed)} instead of the floor spring 0/-185 velocity`,
  );
  semanticAssert(
    dash.dashes === 0,
    "dash-spring-cancel",
    "buffered Dash did not spend the spring-refilled dash",
  );
}
