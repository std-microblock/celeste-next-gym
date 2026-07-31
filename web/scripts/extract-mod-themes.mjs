import { mkdir, readFile, writeFile, readdir, stat, unlink, rename } from "node:fs/promises";
import { mkdirSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import sharp from "sharp";
import { parseCelesteBin, attrText } from "./celeste-bin.mjs";

/**
 * Extract one switchable theme per map from a Celeste mod pack: tileset +
 * autotiler rules, spike type, spinner, and parallax background layers.
 * Chinese map names are read from the mod's Dialog/Simplified Chinese.txt.
 *
 * usage: node scripts/extract-mod-themes.mjs <mod-root|zip> <out-dir>
 *          [--filter globs] [--atlas existing.json,existing.json] [--meta meta.json] [--sid Prefix]
 */
const args = process.argv.slice(2);
const input = args[0];
const outDir = args[1];
if (!input || !outDir) {
  console.error("usage: node scripts/extract-mod-themes.mjs <mod-root|zip> <out-dir> [--filter globs] [--atlas json] [--meta json] [--sid Prefix] [--base-atlas json] [--base-fg xml]");
  process.exit(2);
}
const takeArg = (name) => {
  const index = args.indexOf(name);
  return index !== -1 ? args[index + 1] : undefined;
};
const filterArg = takeArg("--filter");
const filters = filterArg ? filterArg.split(",").filter(Boolean) : null;
const existingAtlas = takeArg("--atlas");
const metaPath = takeArg("--meta");
const sidOverride = takeArg("--sid");
const baseAtlas = takeArg("--base-atlas");
const baseFg = takeArg("--base-fg");

const modRoot = await resolveModRoot(input);
const gameplayRoot = path.join(modRoot, "Graphics", "Atlases", "Gameplay");
const mapsRoot = path.join(modRoot, "Maps");

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
  xmlText = xmlText.replace(/<!--[\s\S]*?-->/g, "");
  const tilesets = {};
  const tagRe = /<Tileset\s+([^>]*?)\/?>/gs;
  for (const match of xmlText.matchAll(tagRe)) {
    const attrs = match[1];
    const id = /id\s*=\s*"([^"]+)"/.exec(attrs)?.[1];
    if (!id) continue;
    const scanW = parseInt(/scanWidth\s*=\s*"(\d+)"/.exec(attrs)?.[1] ?? "3", 10) || 3;
    const scanH = parseInt(/scanHeight\s*=\s*"(\d+)"/.exec(attrs)?.[1] ?? "3", 10) || 3;
    tilesets[id] = {
      id,
      path: /path\s*=\s*"([^"]+)"/.exec(attrs)?.[1] ?? null,
      copy: /copy\s*=\s*"([^"]+)"/.exec(attrs)?.[1] ?? null,
      scan: (scanW !== 3 || scanH !== 3) ? [scanW, scanH] : null,
      ignores: /ignores\s*=\s*"([^"]+)"/.exec(attrs)?.[1] ?? "",
      rules: [],
      center: null,
      padded: null,
    };
  }
  const bodyRe = /<Tileset\s+([^>]*?)(?<!\/)>([\s\S]*?)<\/Tileset>/g;
  for (const match of xmlText.matchAll(bodyRe)) {
    const id = /id\s*=\s*"([^"]+)"/.exec(match[1])?.[1];
    if (!id || !tilesets[id]) continue;
    const entry = tilesets[id];
    const takeFirst = (tiles) => {
      const first = tiles.split(";")[0].trim().split(",").map(Number);
      return first.length === 2 && Number.isInteger(first[0]) && Number.isInteger(first[1])
        ? [first[0], first[1]]
        : null;
    };
    for (const rule of match[2].matchAll(/<set mask="([^"]+)" tiles="([^"]+)"[^>]*\/?>/g)) {
      if (rule[1] === "center") {
        if (!entry.center) entry.center = takeFirst(rule[2]);
        continue;
      }
      if (rule[1] === "padding") {
        if (!entry.padded) entry.padded = takeFirst(rule[2]);
        continue;
      }
      if (rule[1].startsWith("fill")) continue;
      const first = takeFirst(rule[2]);
      if (first) entry.rules.push([rule[1], first]);
    }
  }
  const maskScore = (mask) => {
    let letters = 0;
    let ys = 0;
    let xs = 0;
    for (const ch of mask.replaceAll("-", "")) {
      if (/[a-zA-Z]/.test(ch)) letters += 1;
      else if (ch === "y" || ch === "Y") ys += 1;
      else if (ch === "x" || ch === "X") xs += 1;
    }
    return [letters, ys, xs];
  };
  const resolveField = (tileset, field, seen) => {
    if (seen.has(tileset.id)) return null;
    seen.add(tileset.id);
    const own = tileset[field];
    if (field === "rules") {
      const copied = tileset.copy && tilesets[tileset.copy]
        ? resolveField(tilesets[tileset.copy], "rules", seen) ?? []
        : [];
      const merged = [...own, ...copied];
      if (tileset.copy) merged.sort((a, b) => {
        const sa = maskScore(a[0]);
        const sb = maskScore(b[0]);
        for (let i = 0; i < 3; i += 1) if (sa[i] !== sb[i]) return sa[i] - sb[i];
        return 0;
      });
      return merged;
    }
    if (own !== null && own !== undefined && own !== "") return own;
    if (tileset.copy && tilesets[tileset.copy]) return resolveField(tilesets[tileset.copy], field, seen);
    return null;
  };
  return Object.values(tilesets).map((tileset) => ({
    id: tileset.id,
    path: tileset.path,
    scan: resolveField(tileset, "scan", new Set()) ?? [3, 3],
    ignores: tileset.ignores,
    rules: resolveField(tileset, "rules", new Set()) ?? [],
    center: resolveField(tileset, "center", new Set()),
    padded: resolveField(tileset, "padded", new Set()),
  }));
}

// Chinese map names from Dialog/Simplified Chinese.txt (UTF-8 or GBK).
async function readDialogNames(root) {
  for (const file of ["Dialog/Simplified Chinese.txt", "Dialog/chinese.txt"]) {
    let buf;
    try { buf = await readFile(path.join(root, file)); } catch { continue; }
    let text = "";
    try { text = new TextDecoder("utf-8", { fatal: true }).decode(buf); }
    catch { text = new TextDecoder("gbk").decode(buf); }
    const names = {};
    let current = null;
    for (const raw of text.split(/\r?\n/)) {
      const line = raw.trim();
      if (!line) continue;
      const keyMatch = /^([A-Za-z0-9_.-]+)=/.exec(line);
      if (keyMatch) {
        current = keyMatch[1];
        names[current] = line.slice(line.indexOf("=") + 1).trim();
      } else if (current) {
        names[current] += (names[current] ? "\n" : "") + line;
      }
    }
    return names;
  }
  return {};
}
const dialogNames = await readDialogNames(modRoot);
const modId = sidOverride ?? (await readdir(mapsRoot))[0] ?? "Mod";

const xmlFiles = await findFiles(modRoot, "ForegroundTiles.xml");
const xmlTables = new Map();
for (const xmlFile of xmlFiles) {
  const relative = path.relative(modRoot, xmlFile).replaceAll("\\", "/");
  xmlTables.set(relative, parseForegroundTiles(await readFile(xmlFile, "utf8")));
}
if (baseFg) {
  // Maps without a ForegroundTiles meta attribute fall back to the vanilla
  // game's default Graphics/ForegroundTiles.xml (which the web does not ship).
  xmlTables.set("Graphics/ForegroundTiles.xml", parseForegroundTiles(await readFile(baseFg, "utf8")));
}

function globMatch(pattern, room) {
  const escaped = pattern.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return new RegExp("^" + escaped.replace(/\\\*/g, ".*") + "$").test(room);
}

function backdropVisible(backdrop, room) {
  const attributes = backdrop.attributes;
  if (typeof attributes.exclude === "string" && attributes.exclude.split(",").some((p) => p && globMatch(p, room))) return false;
  if (attributes.flag) return false;
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
  const texture = attr("texture");
  if (typeof texture !== "string") return null;
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

async function addSpikeTextures(spike) {
  const parts = spike.split("/");
  const base = parts.pop();
  const dir = path.join(gameplayRoot, "danger", "spikes", ...parts);
  let files = [];
  try { files = await readdir(dir); } catch { return; }
  for (const name of files) {
    if (name.startsWith(base + "_") && name.endsWith(".png")) {
      referencedTextures.add("danger/spikes/" + (parts.length ? parts.join("/") + "/" : "") + name.slice(0, -4));
    }
  }
}

const referencedTextures = new Set();
const rooms = [];
const metaOverrides = metaPath ? JSON.parse(await readFile(metaPath, "utf8")) : {};

function i18nLabel(mapRelative) {
  let segments = mapRelative.replace(/\.bin$/, "").split("/");
  if (segments[0] === modId) segments = segments.slice(1);
  const candidates = [];
  if (segments[0] === "0-Gyms") {
    candidates.push(modId + "_" + segments.slice(1).map((s) => s.replace(/[^A-Za-z0-9]/g, "_")).join("_"));
  }
  candidates.push(modId + "_" + segments.map((s) => s.replace(/[^A-Za-z0-9]/g, "_")).join("_"));
  for (const key of candidates) {
    const name = dialogNames[key];
    if (name && name.trim()) return name.trim().split("\n")[0];
  }
  return null;
}

const mapFiles = (await findFiles(mapsRoot, ".bin")).filter((file) => {
  if (!filters) return true;
  const relative = path.relative(mapsRoot, file).replaceAll("\\", "/");
  return filters.some((pattern) => {
    const escaped = pattern.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    return new RegExp("^" + escaped.replace(/\\\*/g, ".*") + "$").test(relative);
  });
});

async function tilesetTable(fgPath) {
  if (!xmlTables.has(fgPath)) {
    try {
      const buf = await readFile(path.join(modRoot, fgPath));
      xmlTables.set(fgPath, parseForegroundTiles(buf.toString("utf8")));
    } catch {
      // leave the table undefined (map falls back to no tileset)
    }
  }
  return xmlTables.get(fgPath);
}

for (const mapFile of mapFiles) {
  const root = parseCelesteBin(await readFile(mapFile));
  if (!root) {
    console.warn("skip unparseable map", mapFile);
    continue;
  }
  const meta = root.children.find((c) => c.name === "meta");
  const style = root.children.find((c) => c.name === "Style");
  const fgPath = (attrText(meta, "ForegroundTiles") ?? "Graphics/ForegroundTiles.xml").replaceAll("\\", "/");
  const tilesets = await tilesetTable(fgPath);
  const mapRelative = path.relative(mapsRoot, mapFile).replaceAll("\\", "/");
  const mapMeta = metaOverrides[mapRelative] ?? {};
  const levels = root.children.find((c) => c.name === "levels");
  const mapRooms = [];
  for (const level of levels?.children ?? []) {
    const roomName = attrText(level, "name") ?? "unknown";
    const solids = level.children.find((c) => c.name === "solids");
    const grid = attrText(solids, "innerText") ?? "";
    const solidChars = grid.replace(/[\s0]+/g, "");
    const counts = {};
    for (const ch of solidChars) counts[ch] = (counts[ch] ?? 0) + 1;
    const dominantId = Object.entries(counts).sort((a, b) => b[1] - a[1])[0]?.[0];
    const entities = level.children.find((c) => c.name === "entities");
    // Every tile char used by the room, including entity tiletypes (falling
    // blocks, exit blocks, ...) so GenerateBox skins resolve per char.
    const usedChars = new Set(Object.keys(counts));
    for (const entity of entities?.children ?? []) {
      const tiletype = attrText(entity, "tiletype") ?? attrText(entity, "tileType");
      if (tiletype && tiletype.length === 1 && tiletype !== "0") usedChars.add(tiletype);
    }
    const tilesetById = (id) => tilesets?.find((t) => t.id === id);
    const themeTileset = (id) => {
      const used = tilesetById(id);
      if (!used || !used.path) return null;
      return {
        path: used.path.startsWith("tilesets/") ? used.path : "tilesets/" + used.path,
        scan: used.scan,
        ignores: used.ignores,
        rules: used.rules,
        center: used.center,
        padded: used.padded,
      };
    };
    const dominant = themeTileset(dominantId) ?? themeTileset(Object.keys(counts)[0]);
    const byChar = {};
    for (const ch of usedChars) {
      const def = themeTileset(ch);
      if (def) byChar[ch] = def;
    }
    if (dominant?.path) referencedTextures.add(dominant.path);
    for (const def of Object.values(byChar)) {
      if (def.path) referencedTextures.add(def.path);
    }
    const spikeCounts = {};
    for (const entity of entities?.children ?? []) {
      if (!["spikesUp", "spikesDown", "spikesLeft", "spikesRight"].includes(entity.name)) continue;
      const type = attrText(entity, "type") || "default";
      spikeCounts[type] = (spikeCounts[type] ?? 0) + 1;
    }
    const spikeEntries = Object.entries(spikeCounts).sort((a, b) => b[1] - a[1]);
    const detectedSpike = spikeEntries.find(([type]) => type !== "default")?.[0] ?? "default";
    const spike = mapMeta.spike ?? detectedSpike;
    const spinnerBases = new Set();
    for (const entity of entities?.children ?? []) {
      if (entity.name === "VivHelper/CustomSpinner") {
        const dir = attrText(entity, "Directory");
        const sub = attrText(entity, "Subdirectory");
        if (dir) spinnerBases.add(sub ? dir + "/fg_" + sub : dir + "/fg");
      } else if (entity.name === "FrostHelper/IceSpinner") {
        const dir = attrText(entity, "directory");
        if (dir && dir !== "danger/crystal") spinnerBases.add(dir + "/fg");
      }
    }
    const layers = roomBackdrops(style, roomName);
    for (const layer of layers) referencedTextures.add(layer.key);
    if (spike && spike !== "default") await addSpikeTextures(spike);
    mapRooms.push({
      roomName,
      solidsCount: solidChars.length,
      tileset: dominant?.path ?? null,
      tileRules: dominant?.rules ?? null,
      tileScan: dominant?.scan ?? null,
      tileIgnores: dominant?.ignores ?? "",
      centerTile: dominant?.center ?? null,
      paddedTile: dominant?.padded ?? null,
      tilesets: byChar,
      spike,
      spinnerBases: [...spinnerBases],
      layers,
    });
  }

  mapRooms.sort((a, b) => b.solidsCount - a.solidsCount);
  const chosen = mapRooms[0];
  if (!chosen) {
    console.log("extracted " + mapRelative + " (no solid rooms)");
    continue;
  }
  const roomId = mapRelative.replace(/\.bin$/, "").replace(/[^A-Za-z0-9_-]+/g, "-").replace(/^-+|-+$/g, "");
  const label = i18nLabel(mapRelative) ?? (mapMeta.label ? mapMeta.label.replace(/\.bin$/i, "") : null) ?? mapRelative.replace(/\.bin$/, "");
  rooms.push({
    id: roomId,
    label,
    mapFile: mapRelative,
    room: chosen.roomName,
    rooms: mapRooms.map((room) => room.roomName),
    roomCount: mapRooms.length,
    tileset: chosen.tileset,
    tileRules: chosen.tileRules,
    tileScan: chosen.tileScan,
    tileIgnores: chosen.tileIgnores,
    centerTile: chosen.centerTile,
    paddedTile: chosen.paddedTile,
    tilesets: chosen.tilesets,
    spike: chosen.spike,
    spinner: mapMeta.spinner ?? null,
    spinnerBases: chosen.spinnerBases ?? [],
    background: "#000000",
    layers: chosen.layers,
  });
  console.log("extracted " + mapRelative);
}

// ---- per-theme on-demand atlases + preview thumbnails ----------------------
const baseEntries = {};
if (baseAtlas) {
  for (const file of baseAtlas.split(",")) {
    Object.assign(baseEntries, JSON.parse(await readFile(file.trim(), "utf8")).entries);
  }
}

async function fileExistsInMod(key) {
  try {
    return (await stat(path.join(gameplayRoot, ...key.split("/")) + ".png")).isFile();
  } catch {
    return false;
  }
}

async function spikeTextureKeys(spike) {
  const parts = spike.split("/");
  const base = parts.pop();
  const dir = path.join(gameplayRoot, "danger", "spikes", ...parts);
  const keys = [];
  let files = [];
  try { files = await readdir(dir); } catch { return keys; }
  for (const name of files) {
    if (name.startsWith(base + "_") && name.endsWith(".png")) {
      keys.push("danger/spikes/" + (parts.length ? parts.join("/") + "/" : "") + name.slice(0, -4));
    }
  }
  return keys;
}

async function themeTextureKeys(room) {
  const keys = new Set();
  if (room.tilesets) {
    for (const def of Object.values(room.tilesets)) if (def.path) keys.add(def.path);
  } else if (room.tileset) {
    keys.add(room.tileset);
  }
  for (const layer of room.layers) keys.add(layer.key);
  if (room.spike && room.spike !== "default") {
    for (const key of await spikeTextureKeys(room.spike)) keys.add(key);
  }
  if (room.spinner) {
    keys.add(room.spinner.foreground);
    keys.add(room.spinner.background);
  }
  if (room.spinnerBases && room.spinnerBases.length > 0) {
    for (const base of room.spinnerBases) {
      const bgBase = base.replace(/\/fg_|\/fg$/, (m) => m.replace("fg", "bg"));
      for (const prefix of [base, bgBase]) {
        const parts = prefix.split("/");
        const name = parts.pop();
        let files = [];
        try { files = await readdir(path.join(gameplayRoot, ...parts)); } catch { continue; }
        for (const file of files) {
          if (file.startsWith(name) && file.endsWith(".png")) {
            keys.add([...parts, file.slice(0, -4)].join("/"));
          }
        }
      }
    }
  }
  return keys;
}

const modIdOut = path.basename(outDir);
const padding = 2;
for (const room of rooms) {
  // Pack the theme's own textures (not already in the shared base atlas) into
  // a small per-theme atlas so the web only fetches what the selected theme
  // actually needs.
  const packKeys = [];
  for (const key of await themeTextureKeys(room)) {
    if (baseEntries[key]) continue;
    if (await fileExistsInMod(key)) packKeys.push(key);
  }
  let sheetWidth = 1024;
  for (const key of packKeys) {
    const probe = await sharp(path.join(gameplayRoot, ...key.split("/")) + ".png").metadata();
    sheetWidth = Math.max(sheetWidth, probe.width ?? 0);
  }
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
  const themeDir = path.join(outDir, "themes");
  await mkdir(themeDir, { recursive: true });
  if (placements.length > 0) {
    room.atlas = "assets/" + modIdOut + "/themes/" + room.id + ".json";
    const sheetHeight = cursorY + rowHeight + padding;
    const themePng = path.join(themeDir, room.id + ".png");
    await unlink(themePng).catch(() => {});
    await writeFile(
      themePng,
      await sharp({ create: { width: sheetWidth, height: sheetHeight, channels: 4, background: { r: 0, g: 0, b: 0, alpha: 0 } } })
        .composite(await Promise.all(placements.map(async (entry) => ({ input: await readFile(entry.input), left: entry.x, top: entry.y }))))
        .png({ compressionLevel: 9 })
        .toBuffer(),
    );
    const entries = Object.fromEntries(
      placements.map((entry) => [entry.key, { x: entry.x, y: entry.y, width: entry.width, height: entry.height, drawOffsetX: 0, drawOffsetY: 0, frameWidth: entry.width, frameHeight: entry.height }]),
    );
    const themeJson = path.join(themeDir, room.id + ".json");
    const themeJsonTmp = themeJson + ".tmp";
    await writeFile(themeJsonTmp, JSON.stringify({ entries }, null, 2) + "\n", "utf8");
    await rename(themeJsonTmp, themeJson);
  }
  await renderPreview(room);
}

// Find a texture either as a loose mod PNG or inside the shared base atlas.
async function textureSource(key) {
  const loose = path.join(gameplayRoot, ...key.split("/")) + ".png";
  try {
    if ((await stat(loose)).isFile()) return { file: loose, entry: null };
  } catch {
    // fall through
  }
  const entry = baseEntries[key];
  if (entry && baseAtlas) {
    const firstBase = baseAtlas.split(",")[0].trim();
    return { file: firstBase.replace(".json", ".png"), entry };
  }
  return null;
}

async function renderPreview(room) {
  const width = 144;
  const height = 81;
  const parseHex = (hex) => ({
    r: parseInt(hex.slice(1, 3), 16),
    g: parseInt(hex.slice(3, 5), 16),
    b: parseInt(hex.slice(5, 7), 16),
    alpha: 1,
  });
  const composites = [];
  const firstLayer = room.layers.find((layer) => layer.key);
  if (firstLayer) {
    const source = await textureSource(firstLayer.key);
    if (source) {
      if (source.entry) {
        const cropped = await sharp(source.file)
          .extract({ left: source.entry.x, top: source.entry.y, width: source.entry.width, height: source.entry.height })
          .resize(width, height, { fit: "cover" })
          .png()
          .toBuffer();
        composites.push({ input: cropped, left: 0, top: 0 });
      } else {
        const resized = await sharp(source.file).resize(width, height, { fit: "cover" }).png().toBuffer();
        composites.push({ input: resized, left: 0, top: 0 });
      }
    }
  }
  if (room.tileset) {
    const source = await textureSource(room.tileset);
    if (source) {
      const [tileX, tileY] = room.centerTile ?? [2, 15];
      const extract = (left, top, w, h) =>
        source.entry
          ? sharp(source.file).extract({ left: source.entry.x + left, top: source.entry.y + top, width: w, height: h })
          : sharp(source.file).extract({ left, top, width: w, height: h });
      const strip = [];
      for (let x = 0; x < width; x += 12) {
        strip.push(
          extract(tileX * 8, tileY * 8, 8, 8).resize(12, 12).png().toBuffer(),
        );
      }
      const stripBuffers = await Promise.all(strip);
      const bar = await sharp({ create: { width, height: 18, channels: 4, background: { r: 0, g: 0, b: 0, alpha: 0.55 } } })
        .composite(stripBuffers.map((buf, index) => ({ input: buf, left: index * 12, top: 0 })))
        .png()
        .toBuffer();
      composites.push({ input: bar, left: 0, top: height - 18 });
    }
  }
  const previewDir = path.join(outDir, "previews");
  await mkdir(previewDir, { recursive: true });
  room.preview = "assets/" + modIdOut + "/previews/" + room.id + ".png";
  const previewPng = path.join(previewDir, room.id + ".png");
  await unlink(previewPng).catch(() => {});
  await sharp({ create: { width, height, channels: 4, background: parseHex(room.background) } })
    .composite(composites)
    .png({ compressionLevel: 9 })
    .toFile(previewPng);
}

await mkdir(outDir, { recursive: true });
const output = path.join(outDir, "room-themes.json");
const outputTmp = output + ".tmp";
await writeFile(outputTmp, JSON.stringify({ modId, rooms }, null, 2) + "\n", "utf8");
await rename(outputTmp, output);
console.log("wrote " + rooms.length + " map themes (per-theme atlases + previews) to " + output);
