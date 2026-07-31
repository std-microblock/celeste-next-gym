import sharp from "sharp";
import { readFile } from "node:fs/promises";
import { statSync } from "node:fs";
import path from "node:path";
import { autotileCell, type TileSetDef } from "../src/tileRules.ts";

// Renders each extracted room theme's dominant tile layer on a standard solid
// block and reports the opaque pixel ratio, so broken wall themes are easy to
// spot. Supports the SJ 5x5 autotile masks and per-theme on-demand atlases.

const outDir = process.argv[2];
const roomThemes = JSON.parse(await readFile(outDir + "/room-themes.json", "utf8"));

function exists(p) {
  try { return statSync(p).isFile(); } catch { return false; }
}

async function loadAtlas(url) {
  const json = JSON.parse(await readFile(url, "utf8"));
  const raw = await sharp(await readFile(url.replace(".json", ".png"))).raw().toBuffer({ resolveWithObject: true });
  return { entries: json.entries, data: raw.data, width: raw.info.width };
}

const baseAtlas = await loadAtlas("web/public/assets/original/gameplay/gameplay-selected.json");
const mergedCache = new Map();

async function mergedAtlas(room) {
  if (!room.atlas) {
    const entries = {};
    for (const key of Object.keys(baseAtlas.entries)) entries[key] = { sheet: 0, entry: baseAtlas.entries[key] };
    return { entries, sheets: [baseAtlas] };
  }
  if (mergedCache.has(room.id)) return mergedCache.get(room.id);
  const themeAtlas = await loadAtlas(path.join("web/public/assets", room.atlas.replace("assets/", "")));
  const entries = {};
  for (const key of Object.keys(baseAtlas.entries)) entries[key] = { sheet: 0, entry: baseAtlas.entries[key] };
  for (const [key, entry] of Object.entries(themeAtlas.entries)) entries[key] = { sheet: 1, entry };
  const atlas = { entries, sheets: [baseAtlas, themeAtlas] };
  mergedCache.set(room.id, atlas);
  return atlas;
}

function tileOpaqueRatio(atlas, key, tx, ty) {
  const hit = atlas.entries[key];
  if (!hit) return 0;
  const sheet = atlas.sheets[hit.sheet];
  const entry = hit.entry;
  let opaque = 0;
  for (let y = 0; y < 8; y++) {
    for (let x = 0; x < 8; x++) {
      const i = ((entry.y + ty * 8 + y) * sheet.width + (entry.x + tx * 8 + x)) * 4;
      if (sheet.data[i + 3] > 40) opaque++;
    }
  }
  return opaque / 64;
}

const W = 40, H = 22;
const rows = [];

for (const room of roomThemes.rooms) {
  if (!room.tileset) {
    rows.push({ label: room.label, tileset: "(none)", ratio: -1 });
    continue;
  }
  const atlas = await mergedAtlas(room);
  const def = {
    path: room.tileset,
    scan: room.tileScan ?? [3, 3],
    ignores: room.tileIgnores ?? "",
    rules: room.tileRules ?? [],
    center: room.centerTile,
    padded: room.paddedTile,
  };
  let opaque = 0;
  const total = W * H;
  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      const tile = autotileCell(Array.from({ length: H }, () => Array(W).fill("1")), x, y, def) ?? def.padded ?? def.center ?? [2, 15];
      opaque += tileOpaqueRatio(atlas, room.tileset, tile[0], tile[1]);
    }
  }
  rows.push({ label: room.label, tileset: room.tileset, ratio: opaque / total });
}

rows.sort((a, b) => a.ratio - b.ratio);
let low = 0;
for (const r of rows) {
  const pct = r.ratio < 0 ? "SKIP" : (r.ratio * 100).toFixed(1) + "%";
  if (r.ratio >= 0 && r.ratio < 0.98) low++;
  console.log(pct.padStart(6), r.tileset.padEnd(44), r.label);
}
console.log("themes below 98% opaque:", low, "of", rows.filter((r) => r.ratio >= 0).length);
