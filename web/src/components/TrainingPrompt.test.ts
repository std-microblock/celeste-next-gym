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
});
