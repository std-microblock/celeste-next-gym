import { decode, encode } from "@msgpack/msgpack";
import type { GymMap, SimInput, SimState } from "../model";
import init, {
  cache_simulation_map_msgpack,
  decode_celeste_map_msgpack,
  fuzz_search_cached_map_msgpack,
  list_celeste_map_rooms_msgpack,
  simulate_cached_map_msgpack,
  training_entry_check_msgpack,
} from "../wasm/celeste_wasm.js";

type Request =
  | { id: number; type: "ready" }
  | {
      id: number;
      type: "loadMap";
      bytes: ArrayBuffer;
      room: string;
      name: string;
    }
  | { id: number; type: "listMapRooms"; bytes: ArrayBuffer }
  | {
      id: number;
      type: "fuzzSearch";
      state: SimState;
      fuzz: string;
      map?: GymMap;
      mapVersion: number;
    }
  | { id: number; type: "entryCheck"; state: SimState; checks: string[] }
  | {
      id: number;
      type: "simulate";
      state: SimState;
      inputs: SimInput[];
      map?: GymMap;
      mapVersion: number;
    };

const ready = init();
let cachedMapVersion = 0;

function simulationMap(map: GymMap): Omit<GymMap, "name"> {
  return {
    bounds: map.bounds,
    spawn: map.spawn,
    solids: map.solids,
    entities: map.entities,
    source_package: map.source_package,
  };
}

self.onmessage = async (event: MessageEvent<Request>) => {
  const request = event.data;
  try {
    await ready;
    if (request.type === "ready") {
      self.postMessage({ id: request.id, ok: true, value: "ready" });
      return;
    }
    if (request.type === "loadMap") {
      const response = decode(
        new Uint8Array(
          decode_celeste_map_msgpack(
            new Uint8Array(request.bytes),
            request.room,
          ),
        ),
      ) as { map?: Omit<GymMap, "name">; error?: string };
      if (!response.map)
        throw new Error(response.error ?? "WASM 无法解码 Celeste 地图");
      self.postMessage({
        id: request.id,
        ok: true,
        value: { ...response.map, name: request.name },
      });
      return;
    }
    if (request.type === "listMapRooms") {
      const response = decode(
        list_celeste_map_rooms_msgpack(new Uint8Array(request.bytes)),
      ) as { rooms?: string[]; error?: string };
      if (!response.rooms)
        throw new Error(response.error ?? "WASM 无法读取 Celeste 地图房间列表");
      self.postMessage({ id: request.id, ok: true, value: response.rooms });
      return;
    }
    // The entry check is map-independent; all other requests run against the
    // cached map and therefore carry its version.
    if (request.type === "entryCheck") {
      const response = decode(
        training_entry_check_msgpack(
          encode(request.state),
          JSON.stringify(request.checks),
        ),
      ) as boolean | { error?: string };
      if (typeof response !== "boolean")
        throw new Error(response.error ?? "入口检查返回无效结果");
      self.postMessage({ id: request.id, ok: true, value: response });
      return;
    }
    if (request.map) {
      cache_simulation_map_msgpack(encode(simulationMap(request.map)));
      cachedMapVersion = request.mapVersion;
    } else if (request.mapVersion !== cachedMapVersion) {
      throw new Error("WASM Worker 地图缓存版本不匹配");
    }
    if (request.type === "fuzzSearch") {
      const response = decode(
        fuzz_search_cached_map_msgpack(encode(request.state), request.fuzz),
      ) as { candidates?: unknown[]; evaluations?: unknown[]; error?: string };
      if (!response.candidates || !response.evaluations)
        throw new Error(response.error ?? "Fuzz 没有返回候选评估");
      self.postMessage({ id: request.id, ok: true, value: response });
      return;
    }
    const bytes = simulate_cached_map_msgpack(
      encode(request.state),
      encode(request.inputs),
      request.inputs.length,
    );
    const response = decode(bytes) as { states?: SimState[]; error?: string };
    if (!response.states)
      throw new Error(response.error ?? "WASM 返回了无效轨迹");
    self.postMessage({ id: request.id, ok: true, value: response.states });
  } catch (error) {
    self.postMessage({
      id: request.id,
      ok: false,
      error: error instanceof Error ? error.message : String(error),
    });
  }
};
