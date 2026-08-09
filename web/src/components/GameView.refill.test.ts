import { describe, expect, it } from "vitest";
import { centeredAtlasEntryDestination } from "./GameView";

describe("GameView refill atlas sizing", () => {
  it("draws the cropped one-dash crystal at its native 10x10 size", () => {
    expect(
      centeredAtlasEntryDestination(
        {
          width: 10,
          height: 10,
          drawOffsetX: 3,
          drawOffsetY: 3,
          frameWidth: 16,
          frameHeight: 16,
        },
        100,
        80,
      ),
    ).toEqual({ x: 95, y: 75, width: 10, height: 10 });
  });

  it("draws the cropped two-dash crystal at its native 8x12 size", () => {
    expect(
      centeredAtlasEntryDestination(
        {
          width: 8,
          height: 12,
          drawOffsetX: 4,
          drawOffsetY: 2,
          frameWidth: 16,
          frameHeight: 16,
        },
        100,
        80,
      ),
    ).toEqual({ x: 96, y: 74, width: 8, height: 12 });
  });
});
