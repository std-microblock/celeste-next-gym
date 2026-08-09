import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  createPolicySoakBatch,
  createFirstActionProbe,
  createExpertDecisionInputs,
  createSeedTrajectoryInputs,
  createSeededRandom,
  parseActorOptions,
  validateDirectGymResponse,
} from "../../gym-actors.js";

describe("persistent gym actor launcher", () => {
  it("defaults to one hidden repository-owned actor", () => {
    assert.deepEqual(parseActorOptions([]), {
      count: 1,
      areaId: 1,
      areaMode: 0,
      showWindows: false,
      skipPrepareMods: false,
      smoke: false,
      seedSmoke: false,
      seedSmokeSeed: 8675309,
      inputLifecycleSmoke: false,
      inputLifecycleRounds: 100,
      expertReplaySmoke: false,
      expertReplayRounds: 5,
      soakResets: 0,
      soakRoom: "2",
      soakFrames: 1536,
      soakRestartAt: 0,
      soakPolicy: false,
      soakSeed: 1,
      soakActionFrames: 8,
      directTcp: false,
    });
  });

  it("accepts bounded parallel actors and an explicit Mod SID", () => {
    assert.deepEqual(
      parseActorOptions([
        "--actors",
        "4",
        "--area-id",
        "7",
        "--area-mode",
        "2",
        "--area-sid",
        "Example/TrainingMap",
        "--show-windows",
        "--skip-prepare-mods",
        "--smoke",
        "--seed-smoke",
        "--seed-smoke-seed",
        "-2147483648",
        "--input-lifecycle-smoke",
        "--input-lifecycle-rounds",
        "250",
        "--expert-replay-smoke",
        "--expert-replay-rounds",
        "7",
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
        "--direct-tcp",
      ]),
      {
        count: 4,
        areaId: 7,
        areaMode: 2,
        areaSid: "Example/TrainingMap",
        showWindows: true,
        skipPrepareMods: true,
        smoke: true,
        seedSmoke: true,
        seedSmokeSeed: -2147483648,
        inputLifecycleSmoke: true,
        inputLifecycleRounds: 250,
        expertReplaySmoke: true,
        expertReplayRounds: 7,
        soakResets: 10,
        soakRoom: "2",
        soakFrames: 1536,
        soakRestartAt: 5,
        soakPolicy: true,
        soakSeed: 12345,
        soakActionFrames: 12,
        directTcp: true,
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

  it("builds a fixed action-heavy trajectory for the exact seed gate", () => {
    const inputs = createSeedTrajectoryInputs();
    assert.equal(inputs.length, 128);
    assert.ok(inputs.some((input) => input.move_x === -1));
    assert.ok(inputs.some((input) => input.move_x === 1));
    assert.ok(inputs.some((input) => input.jump_pressed));
    assert.ok(inputs.some((input) => input.dash_pressed));
    assert.ok(inputs.some((input) => input.crouch_dash_pressed));
    assert.ok(inputs.some((input) => input.grab_held));
  });

  it("builds distinct first-action probes for persistent reset lifecycle checks", () => {
    const right = createFirstActionProbe("right");
    const jump = createFirstActionProbe("jump");
    const dash = createFirstActionProbe("dash");
    assert.equal(right.length, 4);
    assert.ok(right.every((input) => input.move_x === 1));
    assert.deepEqual(jump.map((input) => input.jump_pressed), [true, false, false, false]);
    assert.ok(jump.every((input) => input.jump_held));
    assert.deepEqual(dash.map((input) => input.dash_pressed), [true, false, false, false]);
  });

  it("maps expert decisions to four held-input physics frames", () => {
    const first = createExpertDecisionInputs([2, 0, 1, 2, 1], false);
    assert.equal(first.length, 4);
    assert.deepEqual(first.map((input) => input.move_x), [1, 1, 1, 1]);
    assert.deepEqual(first.map((input) => input.move_y), [-1, -1, -1, -1]);
    assert.deepEqual(first.map((input) => input.jump_pressed), [true, false, false, false]);
    assert.deepEqual(first.map((input) => input.crouch_dash_pressed), [true, false, false, false]);
    assert.ok(first.every((input) => input.jump_held && input.grab_held));
    const repeatedJump = createExpertDecisionInputs([1, 1, 1, 0, 0], true);
    assert.ok(repeatedJump.every((input) => !input.jump_pressed));
  });

  it("requires Direct TCP gym observations to report an official area mode", () => {
    const result = validateDirectGymResponse({
      success: true,
      observation: { area_id: 4, area_mode: 1, room: "a-00" },
      frames_executed: 0,
      player_states: [],
    });
    assert.equal(result.observation?.area_mode, 1);
    assert.throws(
      () => validateDirectGymResponse({
        success: true,
        observation: { area_id: 4, room: "a-00" },
      }),
      /area_mode is invalid/,
    );
    assert.throws(
      () => validateDirectGymResponse({
        success: true,
        observation: { area_id: 4, area_mode: 3, room: "a-00" },
      }),
      /area_mode is invalid/,
    );
  });

  it("rejects unsafe actor counts and unknown flags", () => {
    assert.throws(() => parseActorOptions(["--actors", "0"]), /1 through 32/);
    assert.throws(() => parseActorOptions(["--actors", "33"]), /1 through 32/);
    assert.throws(() => parseActorOptions(["--area-mode", "-1"]), /0 through 2/);
    assert.throws(() => parseActorOptions(["--area-mode", "3"]), /0 through 2/);
    assert.throws(
      () => parseActorOptions(["--seed-smoke-seed", "2147483648"]),
      /-2147483648 through 2147483647/,
    );
    assert.throws(() => parseActorOptions(["--unknown"]), /unknown gym actor/);
  });
});
