import { describe, expect, it } from "vitest";
import type { GymMap } from "../model";
import { mapPointTargetPercent } from "./TrainingPrompt";

describe("map-anchored training controls", () => {
  it("projects a map point into the letterboxed game viewport", () => {
    const map = {
      bounds: { x: 100, y: 200, width: 100, height: 50 },
    } as GymMap;

    expect(
      mapPointTargetPercent(map, { x: 100, y: 200 }, { width: 200, height: 200 }),
    ).toEqual({ x: 0, y: 25 });
    expect(
      mapPointTargetPercent(map, { x: 150, y: 225 }, { width: 200, height: 200 }),
    ).toEqual({ x: 50, y: 50 });
  });

  it("projects against the active Celeste camera instead of the whole wide map", () => {
    const map = {
      bounds: { x: 0, y: 0, width: 960, height: 270 },
    } as GymMap;
    const viewport = {
      width: 640,
      height: 360,
      camera: { x: 320, y: 45, width: 320, height: 180 },
    };

    expect(mapPointTargetPercent(map, { x: 320, y: 45 }, viewport)).toEqual({
      x: 0,
      y: 0,
    });
    expect(mapPointTargetPercent(map, { x: 480, y: 135 }, viewport)).toEqual({
      x: 50,
      y: 50,
    });
  });
});
