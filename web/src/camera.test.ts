import { describe, expect, it } from "vitest";
import {
  cameraBounds,
  defaultCameraPosition,
  fitCameraViewport,
  stateCameraPosition,
  zoomCameraViewport,
} from "./camera";
import { createInitialState, type GymMap } from "./model";

const wideMap = {
  bounds: { x: 0, y: 0, width: 960, height: 270 },
  spawn: { x: 32, y: 240 },
} as GymMap;

describe("Celeste camera viewport", () => {
  it("uses the simulated top-left camera position without fitting the whole map", () => {
    const state = createInitialState(wideMap);
    state.camera = { x: 417.25, y: 72.5 };
    expect(stateCameraPosition(wideMap, state)).toEqual(state.camera);
    expect(cameraBounds(stateCameraPosition(wideMap, state))).toEqual({
      x: 417.25,
      y: 72.5,
      width: 320,
      height: 180,
    });
  });

  it("centers and clamps the fallback camera on wide maps", () => {
    expect(defaultCameraPosition(wideMap, { x: 32, y: 240 })).toEqual({
      x: 0,
      y: 90,
    });
    expect(defaultCameraPosition(wideMap, { x: 900, y: 120 })).toEqual({
      x: 640,
      y: 30,
    });
    const uninitialized = createInitialState(wideMap);
    uninitialized.pos = { x: 900, y: 120 };
    uninitialized.camera = { x: 0, y: 0 };
    uninitialized.camera_initialized = false;
    expect(stateCameraPosition(wideMap, uninitialized)).toEqual({
      x: 640,
      y: 30,
    });
  });

  it("fits the entire map and zooms around the requested focus point", () => {
    const fitted = fitCameraViewport(wideMap);
    expect(fitted).toEqual({
      x: 0,
      y: -135,
      width: 960,
      height: 540,
    });
    expect(
      zoomCameraViewport(wideMap, fitted, 0.5, { x: 480, y: 135 }),
    ).toEqual({
      x: 240,
      y: 0,
      width: 480,
      height: 270,
    });
  });
});
