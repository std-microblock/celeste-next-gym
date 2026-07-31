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


/** One tileset definition inside a room theme: autotile rules of any scan
 * size (the game supports 3x3 and the SJ 5x5 extension), padding/center
 * interior tiles, and the neighbor-ignore list. */
export interface TileSetDef {
  path?: string;
  scan?: readonly [number, number];
  ignores?: string;
  rules?: readonly (readonly [string, readonly [number, number]])[];
  center?: readonly [number, number];
  padded?: readonly [number, number];
}

function cellChar(grid: string[][], x: number, y: number, edges: "clamp" | "empty"): string {
  if (edges === "empty") return grid[y]?.[x] ?? "0";
  const rows = grid.length;
  const cols = grid[0]?.length ?? 0;
  if (rows === 0 || cols === 0) return "0";
  const cx = Math.max(0, Math.min(cols - 1, x));
  const cy = Math.max(0, Math.min(rows - 1, y));
  return grid[cy]?.[cx] ?? "0";
}

function cellFilled(
  def: TileSetDef,
  grid: string[][],
  x: number,
  y: number,
  edges: "clamp" | "empty",
): boolean {
  const ch = cellChar(grid, x, y, edges);
  if (ch === "0" || ch === " ") return false;
  const ignores = def.ignores ?? "";
  if (ignores.includes("*")) return false;
  return !ignores.includes(ch);
}

/** Port of the (patched) Celeste Autotiler.TileHandler: builds the scan-size
 * neighborhood, matches 0/1/x/y masks in order, and falls back to the
 * padded/center interior tiles for fully-surrounded cells. The edges flag
 * mirrors Behaviour.EdgesExtend: ground clamps out-of-bounds to the edge
 * tile while a GenerateBox (falling block) treats them as empty. */
export function autotileCell(
  grid: string[][],
  x: number,
  y: number,
  def: TileSetDef,
  edges: "clamp" | "empty" = "clamp",
): readonly [number, number] | undefined {
  const scan = def.scan ?? [3, 3];
  const scanW = scan[0];
  const scanH = scan[1];
  const span: string[] = [];
  const spanFilled: boolean[] = [];
  let allFilled = true;
  const midX = Math.floor(scanW / 2);
  const midY = Math.floor(scanH / 2);
  for (let dy = 0; dy < scanH; dy += 1) {
    for (let dx = 0; dx < scanW; dx += 1) {
      const nx = x + dx - midX;
      const ny = y + dy - midY;
      const filled = cellFilled(def, grid, nx, ny, edges);
      span.push(cellChar(grid, nx, ny, edges));
      spanFilled.push(filled);
      if (!filled) allFilled = false;
    }
  }
  const selfChar = span[midY * scanW + midX];
  if (allFilled) {
    const w = 1 + midX;
    const h = 1 + midY;
    const cross =
      cellFilled(def, grid, x - w, y, edges) &&
      cellFilled(def, grid, x + w, y, edges) &&
      cellFilled(def, grid, x, y - h, edges) &&
      cellFilled(def, grid, x, y + h, edges);
    if (cross) return def.center ?? def.padded ?? undefined;
    return def.padded ?? def.center ?? undefined;
  }
  for (const [mask, tile] of def.rules ?? []) {
    const cells = mask.replaceAll("-", "");
    if (cells.length !== scanW * scanH) continue;
    let ok = true;
    for (let k = 0; k < cells.length && ok; k += 1) {
      const ch = cells[k];
      if (ch === "x" || ch === "X" || ch === "z" || ch === "Z") continue;
      if (ch === "0" && spanFilled[k]) ok = false;
      else if (ch === "1" && !spanFilled[k]) ok = false;
      else if ((ch === "y" || ch === "Y") && span[k] === selfChar) ok = false;
      else if (/[a-zA-Z]/.test(ch)) ok = false; // whitelist/blacklist letters unsupported
    }
    if (ok) return [tile[0], tile[1]];
  }
  return undefined;
}

/** FallingBlock skin: GFX.FGAutotiler.GenerateBox(tiletype, w/8, h/8) with the
 * default Behaviour (EdgesExtend=false), so cells outside the box are empty. */
export function autotileBox(
  boxGrid: string[][],
  x: number,
  y: number,
  def: TileSetDef,
): readonly [number, number] {
  return autotileCell(boxGrid, x, y, def, "empty") ?? def.padded ?? def.center ?? [2, 15];
}
export function strawberryJamGymTileCoordinate(
  grid: string[][],
  x: number,
  y: number,
): [number, number] {
  return autotileByRules(grid, x, y, SJ_GYM_TILE_RULES) ?? [2, 15];
}
