import { mkdir, readFile, writeFile, readdir, stat } from "node:fs/promises";
import { mkdirSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import sharp from "sharp";
import { parseCelesteBin, attrText } from "./celeste-bin.mjs";

/**
 * Extract each room's theme (tileset + autotiler rules, spike type, parallax
 * background layers) from a Celeste mod pack into a JSON theme file that the
 * web theme picker can switch to.
 *
 * usage: node scripts/extract-mod-themes.mjs <mod-root|mod.zip> <out-dir>
 *          [--filter 0-Gyms/*,0-Lobbies/*] [--atlas <existing theme-selected.json>]
 */
const args = process.argv.slice(2);
const input = args[0];
const outDir = args[1];
if (!input || !outDir) {
  console.error("usage: node scripts/extract-mod-themes.mjs <mod-root|zip> <out-dir> [--filter globs] [--atlas json]");
  process.exit(2);
}
const filterArg = args[args.indexOf("--filter") + 1];
const filters = filterArg ? filterArg.split(",").filter(Boolean) : null;
const existingAtlas = args[args.indexOf("--atlas") + 1];
const metaPath = args[args.indexOf("--meta") + 1];
const metaOverrides = metaPath ? JSON.parse(await readFile(metaPath, "utf8")) : {};

const modRoot = await resolveModRoot(input);
const gameplayRoot = path.join(modRoot, "Graphics", "Atlases", "Gameplay");

async function resolveModRoot(input) {
  const resolved = path.resolve(input);
  const info = await stat(resolved);
  if (info.isDirectory()) return resolved;
  if (!/\.zip$/i.test(resolved)) throw new Error("unsupported input: " + resolved);
  const target = path.join(tmpdir(), "celeste-mod-" + path.basename(resolved, ".zip") + "-" + Date.now());
  mkdirSync(target, { recursive: true });
  const result = spawnSync("tar", ["-xf", resolved, "-C", target], { stdio: "inherit" });
  if (result.status !== 0) throw new Error("failed to extract mod zip (tar -xf)");
  return target;
}

async function findFiles(root, suffix) {
  const out = [];
  async function walk(dir) {
    let entries;
    try { entries = await readdir(dir); } catch { return; }
    for (const entry of entries) {
      const full = path.join(dir, entry);
      const info = await stat(full);
      if (info.isDirectory()) await walk(full);
      else if (entry.endsWith(suffix)) out.push(full);
    }
  }
  await walk(root);
  return out;
}

function parseForegroundTiles(xmlText) {
  // Handle both paired and self-closing <Tileset ...> elements.
  const tilesets = {};
  const tagRe = /<Tileset\s+([^>]*?)\/?>/gs;
  for (const match of xmlText.matchAll(tagRe)) {
    const attrs = match[1];
    const id = /id="([^"]+)"/.exec(attrs)?.[1];
    if (!id) continue;
    tilesets[id] = {
      id,
      path: /path="([^"]+)"/.exec(attrs)?.[1] ?? null,
      copy: /copy="([^"]+)"/.exec(attrs)?.[1] ?? null,
      rules: [],
      center: null,
    };
  }
  const bodyRe = /<Tileset\s+([^>]*?)(?<!\/)>([\s\S]*?)<\/Tileset>/g;
  for (const match of xmlText.matchAll(bodyRe)) {
    const id = /id="([^"]+)"/.exec(match[1])?.[1];
    if (!id || !tilesets[id]) continue;
    const rules = [];
    for (const rule of match[2].matchAll(/<set mask="([^"]+)" tiles="([^"]+)"[^>]*\/?>/g)) {
      if (rule[1] === "center") {
        const first = rule[2].split(";")[0].trim().split(",").map(Number);
        if (first.length === 2 && Number.isInteger(first[0]) && Number.isInteger(first[1])) {
          tilesets[id].center = [first[0], first[1]];
        }
        continue;
      }
      if (!/^[x01]{3}-[x01]{3}-[x01]{3}$/.test(rule[1])) continue;
      // The game randomizes among semicolon-separated candidates; keep the
      // first for a deterministic web render.
      const first = rule[2].split(";")[0].trim().split(",").map(Number);
      if (first.length === 2 && Number.isInteger(first[0]) && Number.isInteger(first[1])) {
        rules.push([rule[1], [first[0], first[1]]]);
      }
    }
    tilesets[id].rules = rules;
  }
  const resolve = (tileset, seen) => {
    if (seen.has(tileset.id)) return [];
    seen.add(tileset.id);
    if (tileset.rules.length > 0) return tileset.rules;
    if (tileset.copy && tilesets[tileset.copy]) return resolve(tilesets[tileset.copy], seen);
    return [];
  };
  const resolveCenter = (tileset, seen) => {
    if (seen.has(tileset.id)) return null;
    seen.add(tileset.id);
    if (tileset.center) return tileset.center;
    if (tileset.copy && tilesets[tileset.copy]) return resolveCenter(tilesets[tileset.copy], seen);
    return null;
  };
  return Object.values(tilesets).map((tileset) => ({
    id: tileset.id,
    path: tileset.path,
    rules: resolve(tileset, new Set()),
    center: resolveCenter(tileset, new Set()),
  }));
}

const xmlFiles = await findFiles(modRoot, "ForegroundTiles.xml");
const xmlTables = new Map();
for (const xmlFile of xmlFiles) {
  const relative = path.relative(modRoot, xmlFile).replaceAll("\\", "/");
  xmlTables.set(relative, parseForegroundTiles(await readFile(xmlFile, "utf8")));
}

function globMatch(pattern, room) {
  const escaped = pattern.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return new RegExp("^" + escaped.replace(/\\\*/g, ".*") + "$").test(room);
}
function backdropVisible(backdrop, room) {
  const attributes = backdrop.attributes;
  if (typeof attributes.exclude === "string" && attributes.exclude.split(",").some((p) => p && globMatch(p, room))) return false;
  if (attributes.flag) return false; // session flags are not evaluated offline
  if (typeof attributes.only === "string" && attributes.only.trim()) {
    const patterns = attributes.only.split(",").filter(Boolean);
    if (!patterns.some((p) => globMatch(p, room))) return false;
  }
  return true;
}

function parallaxLayer(element, above) {
  const attr = (key) =>
    element.attributes[key] !== undefined
      ? element.attributes[key]
      : above?.attributes?.[key];
  const text = (key) => (typeof attr(key) === "string" ? attr(key) : undefined);
  const number = (key, fallback) => {
    const value = attr(key);
    if (typeof value === "boolean") return fallback;
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : fallback;
  };
  const bool = (key, fallback) => {
    const value = attr(key);
    if (typeof value === "boolean") return value;
    return value === undefined ? fallback : String(value) === "true";
  };
  const texture = text("texture");
  if (!texture) return null;
  return {
    key: texture,
    scrollX: number("scrollx", 1),
    scrollY: number("scrolly", 1),
    loopX: bool("loopx", true),
    loopY: bool("loopy", true),
    speedX: number("speedx", 0),
    speedY: number("speedy", 0),
    opacity: number("alpha", 1),
  };
}

function roomBackdrops(style, room) {
  const backgrounds = style?.children?.find((c) => c.name.toLowerCase() === "backgrounds");
  if (!backgrounds) return [];
  const layers = [];
  for (const child of backgrounds.children) {
    if (child.name.toLowerCase() === "apply") {
      for (const inner of child.children) {
        if (inner.name.toLowerCase() !== "parallax") continue;
        if (!backdropVisible(inner, room)) continue;
        const layer = parallaxLayer(inner, child);
        if (layer) layers.push(layer);
      }
    } else if (child.name.toLowerCase() === "parallax") {
      if (!backdropVisible(child, room)) continue;
      const layer = parallaxLayer(child, undefined);
      if (layer) layers.push(layer);
    }
  }
  return layers;
}

const mapsRoot = path.join(modRoot, "Maps");
const mapFiles = (await findFiles(mapsRoot, ".bin")).filter((file) => {
  if (!filters) return true;
  const relative = path.relative(mapsRoot, file).replaceAll("\\", "/");
  return filters.some((pattern) => {
    const escaped = pattern.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    return new RegExp("^" + escaped.replace(/\\\*/g, ".*") + "$").test(relative);
  });
});

const rooms = [];
const referencedTextures = new Set();
for (const mapFile of mapFiles) {
  const bytes = await readFile(mapFile);
  const root = parseCelesteBin(bytes);
  if (!root) {
    console.warn("skip unparseable map", mapFile);
    continue;
  }
  const meta = root.children.find((c) => c.name === "meta");
  const style = root.children.find((c) => c.name === "Style");
  const fgPath = attrText(meta, "ForegroundTiles");
  const tilesets = fgPath ? xmlTables.get(fgPath) : undefined;
  const mapRelative = path.relative(mapsRoot, mapFile).replaceAll("\\", "/");
  const levels = root.children.find((c) => c.name === "levels");
  for (const level of levels?.children ?? []) {
    const roomName = attrText(level, "name") ?? "unknown";
    const solids = level.children.find((c) => c.name === "solids");
    const grid = attrText(solids, "innerText") ?? "";
    const counts = {};
    for (const ch of grid.replace(/[\s0]+/g, "")) counts[ch] = (counts[ch] ?? 0) + 1;
    const dominantId = Object.entries(counts).sort((a, b) => b[1] - a[1])[0]?.[0];
    let tileset = null;
    let tileRules = null;
    let used = null;
    if (tilesets) {
      used = dominantId
        ? tilesets.find((t) => t.id === dominantId) ??
          tilesets.find((t) => counts[t.id])
        : undefined;
      if (used) {
        tileset = used.path.startsWith("tilesets/") ? used.path : "tilesets/" + used.path;
        tileRules = used.rules;
        if (tileset) referencedTextures.add(tileset);
      }
    }
    const entities = level.children.find((c) => c.name === "entities");
    const spikeCounts = {};
    for (const entity of entities?.children ?? []) {
      if (!["spikesUp", "spikesDown", "spikesLeft", "spikesRight"].includes(entity.name)) continue;
      const type = attrText(entity, "type") || "default";
      spikeCounts[type] = (spikeCounts[type] ?? 0) + 1;
    }
    const spikeEntries = Object.entries(spikeCounts).sort((a, b) => b[1] - a[1]);
    const spike = spikeEntries.find(([type]) => type !== "default")?.[0] ?? "default";
    const layers = roomBackdrops(style, roomName);
    for (const layer of layers) referencedTextures.add(layer.key);
    const roomId = (mapRelative.replace(/[^A-Za-z0-9_-]+/g, "-") + "-" + roomName.replace(/[^A-Za-z0-9_-]+/g, "-")).replace(/^-+|-+$/g, "");
    const mapMeta = metaOverrides[mapRelative] ?? {};
    const label = mapMeta.label
      ? mapMeta.label + (mapMeta.label.endsWith(roomName) ? "" : " / " + roomName)
      : mapRelative + " / " + roomName;
    rooms.push({
      id: roomId,
      label,
      mapFile: mapRelative,
      room: roomName,
      tileset,
      tileRules,
      centerTile: used?.center ?? null,
      spike,
      spinner: mapMeta.spinner ?? null,
      background: "#000000",
      layers,
    });
  }
  console.log("extracted " + mapRelative);
}
if (existingAtlas) {
  // --atlas is accepted for backward compatibility; the mod atlas is still
  // packed below so rooms render the mod's own tileset/bg textures.
  void existingAtlas;
}
{
  const packKeys = [];
  for (const key of referencedTextures) {
    const input = path.join(gameplayRoot, ...key.split("/"));
    try {
      if ((await stat(input + ".png")).isFile()) packKeys.push(key);
    } catch {
      // ignore missing texture files
    }
  }
  let sheetWidth = 1024;
  for (const key of packKeys) {
    const probe = path.join(gameplayRoot, ...key.split("/")) + ".png";
    const probeMeta = await sharp(probe).metadata();
    sheetWidth = Math.max(sheetWidth, probeMeta.width ?? 0);
  }
  const padding = 2;
  let cursorX = padding;
  let cursorY = padding;
  let rowHeight = 0;
  const placements = [];
  for (const key of packKeys) {
    const input = path.join(gameplayRoot, ...key.split("/")) + ".png";
    const metadata = await sharp(input).metadata();
    if (cursorX + metadata.width + padding > sheetWidth) {
      cursorX = padding;
      cursorY += rowHeight + padding;
      rowHeight = 0;
    }
    placements.push({ key, input, x: cursorX, y: cursorY, width: metadata.width, height: metadata.height });
    cursorX += metadata.width + padding;
    rowHeight = Math.max(rowHeight, metadata.height);
  }
  const sheetHeight = cursorY + rowHeight + padding;
  await mkdir(path.join(outDir, "gameplay"), { recursive: true });
  await sharp({ create: { width: sheetWidth, height: sheetHeight, channels: 4, background: { r: 0, g: 0, b: 0, alpha: 0 } } })
    .composite(await Promise.all(placements.map(async (entry) => ({ input: await readFile(entry.input), left: entry.x, top: entry.y }))))
    .png({ compressionLevel: 9 })
    .toFile(path.join(outDir, "gameplay", "room-theme-assets.png"));
  const entries = Object.fromEntries(
    placements.map((entry) => [entry.key, { x: entry.x, y: entry.y, width: entry.width, height: entry.height, drawOffsetX: 0, drawOffsetY: 0, frameWidth: entry.width, frameHeight: entry.height }]),
  );
  await writeFile(path.join(outDir, "gameplay", "room-theme-assets.json"), JSON.stringify({ entries }, null, 2) + "\n", "utf8");
  console.log("packed " + placements.length + " textures into " + path.join(outDir, "gameplay", "room-theme-assets"));
}

await mkdir(outDir, { recursive: true });
const output = path.join(outDir, "room-themes.json");
const atlasField = "assets/" + path.basename(outDir) + "/gameplay/room-theme-assets.json";
await writeFile(output, JSON.stringify({ modId: path.basename(outDir), rooms, atlas: atlasField }, null, 2) + "\n", "utf8");
console.log("wrote " + rooms.length + " room themes to " + output);
