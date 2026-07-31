import { describe, expect, it } from "vitest";
import {
  SJ_GYM_TILE_RULES,
  autotileBox,
  autotileByRules,
  autotileCell,
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

describe("autotileCell multi-size masks", () => {
  const five: {
    scan: readonly [number, number];
    rules: readonly (readonly [string, readonly [number, number]])[];
    center: readonly [number, number];
    padded: readonly [number, number];
  } = {
    scan: [5, 5] as const,
    rules: [
      // mosscairn grayExtended top edge: empty above, filled sides/below.
      ["xxxxx-xx0xx-x111x-xx1xx-xx1xx", [0, 0]],
      // corner top-left: empty above+left, filled right+bottom.
      ["xxxxx-xx0xx-x011x-xx1xx-xxxxx", [0, 11]],
    ],
    center: [5, 14],
    padded: [5, 0],
  };

  it("matches a 5x5 top edge mask", () => {
    const ctx = grid([
      "00000",
      "01110",
      "01110",
      "01110",
      "01110",
    ]);
    expect(autotileCell(ctx, 2, 1, five)).toEqual([0, 0]);
  });

  it("matches a 5x5 corner mask", () => {
    const ctx = grid([
      "00000",
      "00111",
      "00111",
      "00111",
      "00111",
    ]);
    // top-left corner of the filled region at (2,1).
    expect(autotileCell(ctx, 2, 1, five)).toEqual([0, 11]);
  });

  it("returns padded for a cell one tile inside the block edge", () => {
    // 13x13 map with a 9x9 filled region (rows/cols 2..10): the 5x5 scan and
    // the +-3 padding cross-check need a wide margin around the tested cell.
    const ctx = grid([
      "0000000000000",
      "0000000000000",
      "0011111111100",
      "0011111111100",
      "0011111111100",
      "0011111111100",
      "0011111111100",
      "0011111111100",
      "0011111111100",
      "0011111111100",
      "0011111111100",
      "0000000000000",
      "0000000000000",
    ]);
    // deep interior (7,7): cross-check at +-3 still filled -> center.
    expect(autotileCell(ctx, 7, 7, five)).toEqual([5, 14]);
    // (4,4): cross-check left/up lands in the empty margin -> padded.
    expect(autotileCell(ctx, 4, 4, five)).toEqual([5, 0]);
  });
});

describe("autotileBox falling block skins", () => {
  const wood: {
    rules: readonly (readonly [string, readonly [number, number]])[];
    center: readonly [number, number];
    padded: readonly [number, number];
  } = {
    rules: [
      ["x0x-111-x1x", [0, 0]],
      ["x1x-111-x0x", [0, 1]],
      ["x1x-011-x1x", [0, 2]],
      ["x1x-110-x1x", [0, 3]],
      ["x0x-111-x0x", [0, 4]],
      ["x1x-010-x1x", [0, 5]],
      ["x0x-011-x0x", [0, 8]],
      ["x0x-110-x0x", [0, 9]],
      ["x0x-010-x0x", [0, 10]],
      ["x0x-011-x1x", [0, 11]],
      ["x0x-110-x1x", [0, 12]],
      ["x1x-011-x0x", [0, 13]],
      ["x1x-110-x0x", [0, 14]],
      ["111-111-110", [4, 0]],
      ["110-111-111", [4, 1]],
      ["111-111-011", [4, 2]],
      ["011-111-111", [4, 3]],
      ["010-111-010", [4, 12]],
    ],
    center: [5, 12],
    padded: [5, 0],
  };

  it("renders a 3x4 box with border masks and padded inner edge", () => {
    const box = grid(["1111", "1111", "1111"]);
    const topLeft = autotileBox(box, 0, 0, wood);
    const top = autotileBox(box, 1, 0, wood);
    const mid = autotileBox(box, 1, 1, wood);
    const bottomRight = autotileBox(box, 3, 2, wood);
    expect(topLeft).toEqual([0, 11]); // corner top-left
    expect(top).toEqual([0, 0]); // top edge
    expect(mid).toEqual([5, 0]); // padded: 3 rows leave no center cell
    expect(bottomRight).toEqual([0, 14]); // corner bottom-right
  });

  it("fills the interior with the center tile on a 6x6 box", () => {
    const box = grid(["111111", "111111", "111111", "111111", "111111", "111111"]);
    expect(autotileBox(box, 1, 1, wood)).toEqual([5, 0]); // padded inner edge
    expect(autotileBox(box, 2, 2, wood)).toEqual([5, 12]); // center
  });
});
