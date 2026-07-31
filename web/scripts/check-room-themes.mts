import sharp from "sharp";
import { readFile } from "node:fs/promises";
import { statSync } from "node:fs";
import path from "node:path";

// Renders each extracted room theme's tile layer on a standard solid block and
// reports the opaque pixel ratio, so broken wall themes are easy to spot.

const outDir = process.argv[2];
const roomThemes = JSON.parse(await readFile(outDir + "/room-themes.json", "utf8"));

async function loadAtlas(url) {
  const json = JSON.parse(await readFile(url, "utf8"));
  const raw = await sharp(await readFile(url.replace(".json", ".png"))).raw().toBuffer({ resolveWithObject: true });
  return { entries: json.entries, data: raw.data, width: raw.info.width };
}
const atlasFiles = [
  "web/public/assets/original/gameplay/gameplay-selected.json",
  "web/public/assets/strawberry-jam/gameplay/theme-selected.json",
];
if (exists(outDir + "/gameplay/room-theme-assets.json")) {
  atlasFiles.push(outDir + "/gameplay/room-theme-assets.json");
}
const atlasSheets = [];
const merged = {};
for (const file of atlasFiles) {
  const atlas = await loadAtlas(file);
  const sheetIndex = atlasSheets.length;
  atlasSheets.push(atlas);
  for (const [key, entry] of Object.entries(atlas.entries)) {
    merged[key] = { entry, sheet: sheetIndex };
  }
}
function exists(p) {
  try { return statSync(p).isFile(); } catch { return false; }
}

function tileOpaqueRatio(key, tx, ty) {
  const hit = merged[key];
  if (!hit) return 0;
  const sheet = atlasSheets[hit.sheet];
  let opaque = 0;
  for (let y = 0; y < 8; y++) {
    for (let x = 0; x < 8; x++) {
      const i = ((hit.entry.y + ty * 8 + y) * sheet.width + (hit.entry.x + tx * 8 + x)) * 4;
      if (sheet.data[i + 3] > 40) opaque++;
    }
  }
  return opaque / 64;
}

function maskMatch(grid, x, y, mask) {
  const cells = mask.replaceAll("-", "");
  for (let dy = -1; dy <= 1; dy++) {
    for (let dx = -1; dx <= 1; dx++) {
      const ch = cells[(dy + 1) * 3 + (dx + 1)];
      if (ch === "x") continue;
      const filled = grid[y + dy]?.[x + dx] !== undefined && grid[y + dy][x + dx] !== "0";
      if ((ch === "1") !== filled) return false;
    }
  }
  return true;
}

const W = 40, H = 22;
const grid = Array.from({ length: H }, () => Array(W).fill("1"));
const rows = [];

for (const room of roomThemes.rooms) {
  if (!room.tileRules || !room.tileset) {
    rows.push({ label: room.label, tileset: room.tileset ?? "(none)", ratio: -1 });
    continue;
  }
  let opaque = 0;
  const total = W * H;
  for (let y = 0; y < H; y++) {
    for (let x = 0; x < W; x++) {
      let tile = room.centerTile ?? [2, 15];
      for (const [mask, t] of room.tileRules) {
        if (maskMatch(grid, x, y, mask)) { tile = t; break; }
      }
      opaque += tileOpaqueRatio(room.tileset, tile[0], tile[1]);
    }
  }
  rows.push({ label: room.label, tileset: room.tileset, ratio: opaque / total });
}

rows.sort((a, b) => a.ratio - b.ratio);
for (const r of rows) {
  const pct = r.ratio < 0 ? "SKIP" : (r.ratio * 100).toFixed(1) + "%";
  console.log(pct.padStart(6), r.tileset.padEnd(44), r.label);
}
