import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { input, inputFrames } from "../inputs.js";
import { field, near, pickCore, semanticAssert } from "../verify.js";
import { reflectedState } from "./helpers.js";

describe("harness helpers", () => {
  it("creates complete immutable input frames", () => {
    const frames = inputFrames(3, (frame) =>
      input({ move_x: frame === 1 ? 1 : 0 }),
    );
    assert.equal(frames.length, 3);
    assert.equal(frames[1]?.move_x, 1);
    assert.equal(frames[1]?.dash_pressed, false);
    assert.equal(Object.isFrozen(frames[1]), true);
    assert.throws(() => inputFrames(-1), /invalid input frame count/);
  });

  it("uses the pinned 0.01 core tolerance and reflected fields", () => {
    const state = reflectedState({ _frame: 7, speed: [100, -20], dead: true });
    assert.equal(near(10.01, 10), true);
    assert.equal(near(10.011, 10), false);
    assert.equal(field(state, "field3"), 3);
    assert.deepEqual(pickCore(state), {
      frame: 7,
      pos: [19, 144],
      speed: [100, -20],
      state: "Normal",
      facing: "Right",
      dashes: 1,
      stamina: 110,
      on_ground: false,
      ducking: false,
      dead: true,
    });
    assert.throws(
      () => semanticAssert(false, "scenario", "guard"),
      /scenario.*guard/,
    );
  });
});
