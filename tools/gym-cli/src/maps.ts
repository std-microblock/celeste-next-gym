/**
 * Map sources: original Celeste `.bin`, Everest mod `.zip`, simulator map
 * JSON, timeline v2 JSON, the bundled playground, and `vanilla:<name>`.
 */
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { basename, extname, resolve } from "node:path";
import { unzipSync } from "fflate";
import { PLAYGROUND_BIN, VANILLA_MAPS } from "./paths.ts";
import { decodeRoom, encodeMap, listRooms } from "./wasm.ts";
import type { GymMap, RawMap, SimInput, SimState } from "./types.ts";

export interface MapSource {
  kind: "bin" | "zip" | "json" | "timeline" | "playground";
  /** File on disk. */
  path: string;
  /** Path of the `.bin` inside a zip (e.g. `Maps/Foo/1-Bar.bin`). */
  entry?: string;
  /** Map path relative to `Maps/` (used for theme matching), e.g. `1-ForsakenCity.bin`. */
  mapPath: string;
  /** BinaryPacker bytes or MessagePack-encoded simulator map. */
  bytes: Uint8Array;
  /** True when `bytes` is an original BinaryPacker file with rooms. */
  binary: boolean;
  /** Timeline documents also carry a start state and inputs. */
  timeline?: { initial_state?: SimState; inputs?: unknown[] };
}

export interface LoadedRoom {
  source: MapSource;
  room: string;
  /** Simulator map document (round-trips into WASM unchanged). */
  raw: RawMap;
  /** MessagePack of `raw`, ready for simulate/fuzz. */
  bytes: Uint8Array;
  /** Renderer view (entity visuals merged into entities). */
  view: GymMap;
}

export interface ZipMapEntry {
  entry: string;
  mapPath: string;
  size: number;
}

/** List every `Maps/**.bin` inside an Everest mod zip. */
export function listZipMaps(zipPath: string): ZipMapEntry[] {
  const found: ZipMapEntry[] = [];
  unzipSync(readFileSync(zipPath), {
    filter(file) {
      if (/^Maps\/.+\.bin$/i.test(file.name)) {
        found.push({
          entry: file.name,
          mapPath: file.name.replace(/^Maps\//i, ""),
          size: file.originalSize,
        });
      }
      return false;
    },
  });
  return found.sort((a, b) => a.entry.localeCompare(b.entry));
}

function readZipEntry(zipPath: string, entry: string): Uint8Array {
  const files = unzipSync(readFileSync(zipPath), { filter: (file) => file.name === entry });
  const bytes = files[entry];
  if (!bytes) throw new Error(`zip ${zipPath} has no entry ${entry}`);
  return bytes;
}

/** Vanilla map lookup by file name, SID suffix or chapter prefix (`1`, `1H`, `7-Summit`). */
export function findVanillaMap(name: string): string {
  if (!existsSync(VANILLA_MAPS)) {
    throw new Error(`vanilla maps not found at ${VANILLA_MAPS} (unpack the game into vendor/celeste-game)`);
  }
  const files = readdirSync(VANILLA_MAPS).filter((file) => file.endsWith(".bin"));
  const wanted = name.replace(/^Celeste\//, "").replace(/\.bin$/i, "");
  const match =
    files.find((file) => file.slice(0, -4) === wanted) ??
    files.find((file) => file.slice(0, -4).toLowerCase() === wanted.toLowerCase()) ??
    files.find((file) => file.split("-")[0].toLowerCase() === wanted.toLowerCase());
  if (!match) throw new Error(`no vanilla map matches "${name}"; available: ${files.join(", ")}`);
  return resolve(VANILLA_MAPS, match);
}

/** Pick one map inside a zip: exact entry, map path, or unique substring. */
function chooseZipMap(zipPath: string, selector: string | undefined): ZipMapEntry {
  const maps = listZipMaps(zipPath);
  if (maps.length === 0) throw new Error(`zip ${zipPath} contains no Maps/**.bin`);
  if (!selector) {
    if (maps.length === 1) return maps[0];
    throw new Error(
      `zip contains ${maps.length} maps; choose one with --bin <path>:\n` +
        maps.map((map) => `  ${map.mapPath}`).join("\n"),
    );
  }
  const lower = selector.toLowerCase().replace(/\\/g, "/");
  const exact = maps.find(
    (map) =>
      map.entry.toLowerCase() === lower ||
      map.mapPath.toLowerCase() === lower ||
      map.mapPath.toLowerCase() === `${lower}.bin`,
  );
  if (exact) return exact;
  const partial = maps.filter((map) => map.mapPath.toLowerCase().includes(lower));
  if (partial.length === 1) return partial[0];
  throw new Error(
    partial.length === 0
      ? `no map in zip matches "${selector}"`
      : `"${selector}" is ambiguous:\n${partial.map((map) => `  ${map.mapPath}`).join("\n")}`,
  );
}

export interface OpenMapOptions {
  /** Map inside a zip (entry path, path under Maps/, or unique substring). */
  bin?: string;
}

/**
 * Open a map source.
 *  - `playground`                    bundled Mechanics Playground
 *  - `vanilla:1-ForsakenCity` / `vanilla:1H`   repo-owned vanilla install
 *  - `path/to/map.bin`
 *  - `path/to/Mod.zip` (+ `bin` option when it has several maps)
 *  - `path/to/map.json`              simulator Map JSON or timeline v2 (`{version:2,map,initial_state,inputs}`)
 */
export function openMapSource(spec: string, options: OpenMapOptions = {}): MapSource {
  if (spec === "playground") {
    return {
      kind: "playground",
      path: PLAYGROUND_BIN,
      mapPath: "CelesteGymPlayground/Playground.bin",
      bytes: readFileSync(PLAYGROUND_BIN),
      binary: true,
    };
  }
  if (spec.startsWith("vanilla:")) {
    const path = findVanillaMap(spec.slice("vanilla:".length));
    return { kind: "bin", path, mapPath: basename(path), bytes: readFileSync(path), binary: true };
  }
  const path = resolve(spec);
  if (!existsSync(path) || !statSync(path).isFile()) throw new Error(`map source not found: ${spec}`);
  const extension = extname(path).toLowerCase();
  if (extension === ".zip") {
    const chosen = chooseZipMap(path, options.bin);
    return {
      kind: "zip",
      path,
      entry: chosen.entry,
      mapPath: chosen.mapPath,
      bytes: readZipEntry(path, chosen.entry),
      binary: true,
    };
  }
  if (extension === ".json") {
    const document = JSON.parse(readFileSync(path, "utf8")) as Record<string, unknown>;
    if (document.map && typeof document.map === "object") {
      return {
        kind: "timeline",
        path,
        mapPath: basename(path),
        bytes: encodeMap(document.map as RawMap),
        binary: false,
        timeline: {
          initial_state: document.initial_state as SimState | undefined,
          inputs: document.inputs as unknown[] | undefined,
        },
      };
    }
    if (document.bounds && document.solids) {
      return { kind: "json", path, mapPath: basename(path), bytes: encodeMap(document as RawMap), binary: false };
    }
    throw new Error(`${spec}: JSON is neither a simulator map nor a timeline document`);
  }
  // Anything else is treated as a BinaryPacker `.bin`.
  const bytes = readFileSync(path);
  const mapsIndex = path.replace(/\\/g, "/").toLowerCase().lastIndexOf("/maps/");
  const mapPath = mapsIndex >= 0 ? path.replace(/\\/g, "/").slice(mapsIndex + 6) : basename(path);
  return { kind: "bin", path, mapPath, bytes, binary: true };
}

export async function sourceRooms(source: MapSource): Promise<string[]> {
  if (!source.binary) return [""];
  return listRooms(source.bytes);
}

export function rawToView(raw: RawMap, name: string, room: string): GymMap {
  const visuals = raw.entity_visuals ?? [];
  return {
    ...(raw as unknown as GymMap),
    name,
    room,
    entities: (raw.entities ?? []).map((entity, index) => ({
      ...entity,
      ...(visuals[index] ?? {}),
    })) as GymMap["entities"],
  };
}

/** Normalise a user room name: accepts `1`, `lvl_1`. */
export function normaliseRoom(room: string | undefined): string {
  return (room ?? "").replace(/^lvl_/, "");
}

/** Decode one room of a source (default: first room). */
export async function loadRoom(source: MapSource, room?: string): Promise<LoadedRoom> {
  let selected = normaliseRoom(room);
  if (source.binary && selected) {
    const rooms = await listRooms(source.bytes);
    if (!rooms.includes(selected)) {
      const loose = rooms.find((name) => name.toLowerCase() === selected.toLowerCase());
      if (!loose) {
        throw new Error(
          `room "${room}" not found in ${source.mapPath}; rooms: ${rooms.slice(0, 80).join(", ")}${rooms.length > 80 ? ", ..." : ""}`,
        );
      }
      selected = loose;
    }
  }
  const raw = await decodeRoom(source.bytes, selected);
  if (source.binary && !selected) selected = (await listRooms(source.bytes))[0] ?? "";
  return {
    source,
    room: selected,
    raw,
    bytes: encodeMap(raw),
    view: rawToView(raw, source.mapPath, selected),
  };
}

/** Default start state for a room: standing at its spawn, like the web app. */
export function initialState(room: LoadedRoom, at?: { x: number; y: number }): SimState {
  return {
    pos: { ...(at ?? room.raw.spawn) },
    speed: { x: 0, y: 0 },
    state: "Normal",
    facing: true,
    dashes: 1,
    stamina: 110,
    on_ground: false,
    ducking: false,
    can_dream_dash: true,
    dead: false,
    death_freeze_pending: false,
    respawn_frames: 0,
    dash_dir: { x: 0, y: 0 },
  };
}

export type { SimInput };
