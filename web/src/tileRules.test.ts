import { describe, expect, it } from "vitest";
import {
  SJ_GYM_TILE_RULES,
  autotileByRules,
  strawberryJamGymTileCoordinate,
} from "./tileRules";

function grid(rows: string[]): string[][] {
  return rows.map((row) => row.split(""));
}

describe("SJ gym autotiler rules", () => {
  it("ports the 46 pinned mask rules plus padding/center comments", () => {
    expect(SJ_GYM_TILE_RULES.length).toBe(46);
    expect(SJ_GYM_TILE_RULES[0]).toEqual(["x0x-111-111", [1, 2]]);
  });

  it("picks the same corner tiles the game's ForegroundTiles.xml does", () => {
    // A solid block with a bottom-right corner at (2,2): the tile sits with
    // empty top+left and filled bottom+right (mask x0x-011-x11 -> (0,2)).
    const block = [
      "..",
      ".0",
      "0?",
    ];
    // 3x3 context with the corner tile at the center.
    const ctx = grid([
      "00.",
      "011",
      "111",
    ]);
    // center (1,1): above empty, left empty, right filled, below filled.
    expect(autotileByRules(ctx, 1, 1, SJ_GYM_TILE_RULES)).toEqual([0, 2]);
  });

  it("uses the ledge-end corner variant (x0x-011-x10 -> (0,15)) when the tile ends", () => {
    const ctx = grid([
      "000",
      "011",
      "110",
    ]);
    // center (1,1): above empty, left empty, right filled, below-right empty.
    expect(autotileByRules(ctx, 1, 1, SJ_GYM_TILE_RULES)).toEqual([0, 15]);
  });

  it("handles the center fill fallback for fully surrounded tiles", () => {
    const ctx = grid(["111", "111", "111"]);
    expect(autotileByRules(ctx, 1, 1, SJ_GYM_TILE_RULES)).toBeUndefined();
    expect(strawberryJamGymTileCoordinate(ctx, 1, 1)).toEqual([2, 15]);
  });
});
