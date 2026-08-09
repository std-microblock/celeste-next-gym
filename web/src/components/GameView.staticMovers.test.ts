import { describe, expect, it } from "vitest";
import { createInitialState, type GymMap } from "../model";
import { runtimeAttachedEntityBounds } from "./GameView";

function movingSpikeMap(): GymMap {
  return {
    name: "moving spikes",
    bounds: { x: 0, y: 0, width: 320, height: 180 },
    spawn: { x: 200, y: 100 },
    solids: [],
    entities: [
      {
        kind: "moving_solid",
        bounds: { x: 16, y: 100, width: 64, height: 8 },
        direction: { x: 60, y: 0 },
        name: "celesteGymMovingSolid",
      },
      {
        kind: "spikes",
        bounds: { x: 80, y: 100, width: 3, height: 8 },
        direction: { x: 1, y: 0 },
        name: "spikesRight",
      },
    ],
    source_package: null,
  };
}

describe("GameView StaticMover spikes", () => {
  it("keeps spikes at their source offset from a moving solid", () => {
    const map = movingSpikeMap();
    const state = createInitialState(map);
    state.moving_solid_time = 1;

    expect(runtimeAttachedEntityBounds(map, state).get(1)).toEqual({
      x: 140,
      y: 100,
      width: 3,
      height: 8,
    });
  });

  it("hides attached spikes while a move block disables StaticMovers", () => {
    const map = movingSpikeMap();
    map.entities[0] = {
      kind: "move_block",
      bounds: { x: 16, y: 100, width: 64, height: 8 },
      direction: { x: 1, y: 0 },
      name: "moveBlock",
    };
    const state = createInitialState(map);
    state.move_blocks = [
      {
        phase: 4,
        wait_timer: 1,
        speed: 0,
        angle: 0,
        crash_timer: 0,
        crash_reset_timer: 0,
        no_steer_timer: 0,
        position: { x: 64, y: 100 },
        remainder: { x: 0, y: 0 },
        lift_speed: { x: 0, y: 0 },
        start: { x: 16, y: 100 },
        visible: false,
        static_movers_enabled: false,
      },
    ];

    expect(runtimeAttachedEntityBounds(map, state).get(1)?.x).toBe(-1_000_000);
  });

  it("keeps springs at their source offset from a move block", () => {
    const map = movingSpikeMap();
    map.entities = [
      {
        kind: "move_block",
        bounds: { x: 16, y: 100, width: 64, height: 8 },
        direction: { x: 1, y: 0 },
        name: "moveBlock",
      },
      {
        kind: "spring",
        bounds: { x: 32, y: 94, width: 16, height: 6 },
        direction: { x: 0, y: -1 },
        name: "spring",
      },
    ];
    const state = createInitialState(map);
    state.move_blocks = [
      {
        phase: 2,
        wait_timer: 0,
        speed: 60,
        angle: 0,
        crash_timer: 0.15,
        crash_reset_timer: 0.1,
        no_steer_timer: 0.2,
        position: { x: 48, y: 108 },
        remainder: { x: 0, y: 0 },
        lift_speed: { x: 60, y: 0 },
        start: { x: 16, y: 100 },
        visible: true,
        static_movers_enabled: true,
      },
    ];

    expect(runtimeAttachedEntityBounds(map, state).get(1)).toEqual({
      x: 64,
      y: 102,
      width: 16,
      height: 6,
    });
  });
});
