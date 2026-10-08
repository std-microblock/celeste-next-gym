/**
 * Map loadability check: decode every room through the simulator's own
 * decoder, list objects it does not simulate, and run a short idle
 * simulation from each spawn so unsupported states/entities surface early.
 */
import { auditMap, decodeRoom, encodeMap, listRooms, simulateRaw } from "./wasm.ts";
import { initialState, type LoadedRoom, type MapSource, rawToView } from "./maps.ts";
import type { RawMap, SimInput } from "./types.ts";

export type RoomStatus = "ok" | "partial" | "error";

export interface RoomCheck {
  room: string;
  status: RoomStatus;
  bounds?: { x: number; y: number; width: number; height: number };
  spawns?: number;
  solids?: number;
  entities?: Record<string, number>;
  /** Raw names decoded as `unknown` (present in the map, not simulated). */
  unsupportedEntities?: Record<string, number>;
  triggers?: Record<string, number>;
  /** Idle simulation from the spawn. */
  simulation?: { frames: number; finalState: string; dead: boolean; error?: string };
  error?: string;
}

export interface MapCheck {
  map: string;
  source: string;
  status: RoomStatus;
  rooms: number;
  ok: number;
  partial: number;
  errors: number;
  unsupportedEntities: Record<string, number>;
  results: RoomCheck[];
  error?: string;
}

const IDLE: SimInput = {
  move_x: 0,
  move_y: 0,
  jump_pressed: false,
  jump_held: false,
  dash_pressed: false,
  crouch_dash_pressed: false,
  grab_held: false,
  talk_pressed: false,
};

async function checkDecoded(name: string, raw: RawMap, frames: number, triggers?: Record<string, number>): Promise<RoomCheck> {
  const entities: Record<string, number> = {};
  const unsupported: Record<string, number> = {};
  for (const entity of raw.entities ?? []) {
    entities[entity.kind] = (entities[entity.kind] ?? 0) + 1;
    if (entity.kind === "unknown") unsupported[entity.name] = (unsupported[entity.name] ?? 0) + 1;
  }
  const result: RoomCheck = {
    room: name,
    status: Object.keys(unsupported).length ? "partial" : "ok",
    bounds: raw.bounds,
    spawns: raw.room_spawns?.length ?? 1,
    solids: raw.solids?.length ?? 0,
    entities,
    unsupportedEntities: unsupported,
    ...(triggers ? { triggers } : {}),
  };
  if (frames > 0) {
    const loaded = { raw, view: rawToView(raw, "", name) } as LoadedRoom;
    try {
      const states = await simulateRaw(initialState(loaded), Array.from({ length: frames }, () => IDLE), encodeMap(raw));
      const last = states[states.length - 1];
      result.simulation = { frames, finalState: last.state, dead: !!states.find((state) => state.dead) };
    } catch (error) {
      result.status = "error";
      result.simulation = { frames, finalState: "", dead: false, error: (error as Error).message };
    }
  }
  return result;
}

export async function checkMap(
  source: MapSource,
  options: { room?: string; frames?: number } = {},
): Promise<MapCheck> {
  const frames = options.frames ?? 30;
  const results: RoomCheck[] = [];
  const base: Omit<MapCheck, "status" | "rooms" | "ok" | "partial" | "errors" | "unsupportedEntities" | "results"> = {
    map: source.mapPath,
    source: source.entry ? `${source.path}!${source.entry}` : source.path,
  };
  if (!source.binary) {
    try {
      const raw = await decodeRoom(source.bytes, "");
      results.push(await checkDecoded("", raw, frames));
    } catch (error) {
      results.push({ room: "", status: "error", error: (error as Error).message });
    }
  } else {
    let rooms: string[];
    let triggersByRoom = new Map<string, Record<string, number>>();
    try {
      rooms = await listRooms(source.bytes);
      const audit = await auditMap(source.bytes);
      triggersByRoom = new Map(audit.map((room) => [room.name, room.trigger_names]));
    } catch (error) {
      return {
        ...base,
        status: "error",
        rooms: 0,
        ok: 0,
        partial: 0,
        errors: 1,
        unsupportedEntities: {},
        results: [],
        error: (error as Error).message,
      };
    }
    const wanted = options.room?.replace(/^lvl_/, "");
    if (wanted && !rooms.includes(wanted)) throw new Error(`room "${options.room}" not in map; rooms: ${rooms.join(", ")}`);
    for (const room of wanted ? [wanted] : rooms) {
      try {
        const raw = await decodeRoom(source.bytes, room);
        results.push(await checkDecoded(room, raw, frames, triggersByRoom.get(room)));
      } catch (error) {
        results.push({ room, status: "error", error: (error as Error).message });
      }
    }
  }
  const unsupportedEntities: Record<string, number> = {};
  for (const result of results) {
    for (const [name, count] of Object.entries(result.unsupportedEntities ?? {})) {
      unsupportedEntities[name] = (unsupportedEntities[name] ?? 0) + count;
    }
  }
  const count = (status: RoomStatus) => results.filter((result) => result.status === status).length;
  const errors = count("error");
  return {
    ...base,
    status: errors ? "error" : count("partial") ? "partial" : "ok",
    rooms: results.length,
    ok: count("ok"),
    partial: count("partial"),
    errors,
    unsupportedEntities,
    results,
  };
}
