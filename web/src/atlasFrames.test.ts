import { describe, expect, it } from "vitest";
import { atlasFrameKeys } from "./atlasFrames";

describe("atlas frame lookup", () => {
  it("uses an exact atlas key for a single-frame animation", () => {
    expect(
      atlasFrameKeys(
        ["characters/player/idle00", "characters/player/duck"],
        "characters/player/duck",
      ),
    ).toEqual(["characters/player/duck"]);
  });

  it("sorts numbered animation frames numerically", () => {
    expect(
      atlasFrameKeys(
        [
          "characters/player/runSlow10",
          "characters/player/runSlow02",
          "characters/player/runSlow01",
        ],
        "characters/player/runSlow",
      ),
    ).toEqual([
      "characters/player/runSlow01",
      "characters/player/runSlow02",
      "characters/player/runSlow10",
    ]);
  });
});
