// Pinned Strawberry Jam 2021 Beginner Gym autotiler rules, copied verbatim
// from mods/CelesteGymTraining/Graphics/CelesteGymTraining/ForegroundTiles.xml.
// The game's Autotiler tries each mask in order (center cell is the tile
// itself); the first match wins. This replaces the hand-rolled four-neighbor
// mapping, which mis-rotated ledge-end corners (e.g. x0x-011-x10 vs x0x-011-x11).
export const SJ_GYM_TILE_RULES: readonly (readonly [string, readonly [number, number]])[] = [
  ["x0x-111-111", [1, 2]],
  ["111-111-x0x", [1, 4]],
  ["x11-011-x11", [0, 3]],
  ["11x-110-11x", [2, 3]],
  ["x0x-011-x11", [0, 2]],
  ["x0x-110-11x", [2, 2]],
  ["x11-011-x0x", [0, 4]],
  ["11x-110-x0x", [2, 4]],
  ["x0x-111-x0x", [1, 14]],
  ["x1x-010-x1x", [2, 12]],
  ["x0x-010-x1x", [2, 11]],
  ["x1x-010-x0x", [2, 13]],
  ["x0x-011-x0x", [0, 14]],
  ["x0x-110-x0x", [2, 14]],
  ["x0x-011-x10", [0, 15]],
  ["x0x-110-01x", [1, 15]],
  ["x10-011-x0x", [0, 16]],
  ["01x-110-x0x", [1, 16]],
  ["x0x-010-x0x", [1, 13]],
  ["110-111-111", [1, 1]],
  ["011-111-111", [2, 1]],
  ["111-111-110", [1, 0]],
  ["111-111-011", [2, 0]],
  ["010-111-111", [0, 11]],
  ["111-111-010", [1, 12]],
  ["110-111-110", [1, 11]],
  ["011-111-011", [0, 12]],
  ["110-111-011", [2, 9]],
  ["011-111-110", [2, 10]],
  ["110-111-010", [0, 9]],
  ["011-111-010", [1, 9]],
  ["010-111-110", [0, 10]],
  ["010-111-011", [1, 10]],
  ["010-111-010", [0, 13]],
  ["x0x-111-010", [2, 7]],
  ["010-111-x0x", [2, 8]],
  ["x10-011-x10", [2, 5]],
  ["01x-110-01x", [2, 6]],
  ["x11-011-x10", [0, 5]],
  ["x10-011-x11", [0, 6]],
  ["11x-110-01x", [1, 5]],
  ["01x-110-11x", [1, 6]],
  ["x0x-111-110", [0, 7]],
  ["x0x-111-011", [1, 7]],
  ["110-111-x0x", [0, 8]],
  ["011-111-x0x", [1, 8]],
  // padding (1,3) is never matched by the mask table; center is the fallback.
];

export function maskMatchesGrid(
  grid: string[][],
  x: number,
  y: number,
  mask: string,
): boolean {
  for (let dy = -1; dy <= 1; dy += 1) {
    for (let dx = -1; dx <= 1; dx += 1) {
      // Masks are stored as "aaa-bbb-ccc"; strip the dashes for index math.
      const ch = mask.replaceAll("-", "")[(dy + 1) * 3 + (dx + 1)];
      if (ch === "x") continue;
      const filled =
        grid[y + dy]?.[x + dx] !== undefined && grid[y + dy][x + dx] !== "0";
      if ((ch === "1") !== filled) return false;
    }
  }
  return true;
}

export function autotileByRules(
  grid: string[][],
  x: number,
  y: number,
  rules: readonly (readonly [string, readonly [number, number]])[],
): [number, number] | undefined {
  for (const [mask, tile] of rules) {
    if (maskMatchesGrid(grid, x, y, mask)) return [tile[0], tile[1]];
  }
  return undefined;
}

export function strawberryJamGymTileCoordinate(
  grid: string[][],
  x: number,
  y: number,
): [number, number] {
  return autotileByRules(grid, x, y, SJ_GYM_TILE_RULES) ?? [2, 15];
}
