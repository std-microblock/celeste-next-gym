import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  createPolicySoakBatch,
  createSeededRandom,
  parseActorOptions,
} from "../../gym-actors.js";

describe("persistent gym actor launcher", () => {
  it("defaults to one hidden repository-owned actor", () => {
    assert.deepEqual(parseActorOptions([]), {
      count: 1,
      areaId: 1,
      showWindows: false,
      smoke: false,
      soakResets: 0,
      soakRoom: "2",
      soakFrames: 1536,
      soakRestartAt: 0,
      soakPolicy: false,
      soakSeed: 1,
      soakActionFrames: 8,
    });
  });

  it("accepts bounded parallel actors and an explicit Mod SID", () => {
    assert.deepEqual(
      parseActorOptions([
        "--actors",
        "4",
        "--area-id",
        "7",
        "--area-sid",
        "Example/TrainingMap",
        "--show-windows",
        "--smoke",
        "--soak-resets",
        "10",
        "--soak-room",
        "2",
        "--soak-frames",
        "1536",
        "--soak-restart-at",
        "5",
        "--soak-policy",
        "--soak-seed",
        "12345",
        "--soak-action-frames",
        "12",
      ]),
      {
        count: 4,
        areaId: 7,
        areaSid: "Example/TrainingMap",
        showWindows: true,
        smoke: true,
        soakResets: 10,
        soakRoom: "2",
        soakFrames: 1536,
        soakRestartAt: 5,
        soakPolicy: true,
        soakSeed: 12345,
        soakActionFrames: 12,
      },
    );
  });

  it("generates reproducible short policy-like action batches", () => {
    const first = createSeededRandom(42);
    const second = createSeededRandom(42);
    const firstBatches = Array.from({ length: 8 }, (_, index) =>
      createPolicySoakBatch(first, 8, index),
    );
    const secondBatches = Array.from({ length: 8 }, (_, index) =>
      createPolicySoakBatch(second, 8, index),
    );
    assert.deepEqual(firstBatches, secondBatches);
    assert.ok(firstBatches.every((batch) => batch.length >= 1 && batch.length <= 8));
    assert.ok(firstBatches.flat().some((input) => input.move_x === -1));
    assert.ok(firstBatches.flat().some((input) => input.move_x === 1));
    assert.ok(firstBatches.flat().some((input) => input.jump_pressed));
    assert.ok(firstBatches.flat().some((input) => input.dash_pressed));
    assert.ok(firstBatches.flat().some((input) => input.crouch_dash_pressed));
    assert.ok(firstBatches.flat().some((input) => input.grab_held));
  });

  it("rejects unsafe actor counts and unknown flags", () => {
    assert.throws(() => parseActorOptions(["--actors", "0"]), /1 through 32/);
    assert.throws(() => parseActorOptions(["--actors", "33"]), /1 through 32/);
    assert.throws(() => parseActorOptions(["--unknown"]), /unknown gym actor/);
  });
});
