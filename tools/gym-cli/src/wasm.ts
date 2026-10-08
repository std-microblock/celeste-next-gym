/**
 * Node binding for the Rust `celeste-wasm` simulator bundle.
 *
 * The bundle is produced by `node scripts/build-wasm.mjs` into
 * `web/src/wasm/` (the same artifact the web app uses). Everything crosses the
 * boundary as MessagePack, exactly like the browser Worker.
 */
import { readFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { decode, encode } from "@msgpack/msgpack";
import { REPO_ROOT } from "./paths.ts";
import type { RawMap, RoomAudit, SimInput, SimState } from "./types.ts";
import type { FuzzRunResult, FuzzSpec } from "./fuzzTypes.ts";

interface WasmModule {
  default(input: { module_or_path: Uint8Array }): Promise<unknown>;
  audit_celeste_map_msgpack(bytes: Uint8Array): Uint8Array;
  decode_celeste_map_msgpack(bytes: Uint8Array, room: string): Uint8Array;
  list_celeste_map_rooms_msgpack(bytes: Uint8Array): Uint8Array;
  simulate_msgpack(
    snapshot: Uint8Array,
    inputs: Uint8Array,
    map: Uint8Array,
    frames: number,
  ): Uint8Array;
  fuzz_search_msgpack(
    snapshot: Uint8Array,
    map: Uint8Array,
    fuzz: string,
    maxCandidates: number,
  ): Uint8Array;
  fuzz_resolve_inputs_msgpack(fuzz: string, bindings: string): Uint8Array;
  training_entry_check_msgpack(snapshot: Uint8Array, checks: string): Uint8Array;
}

let loaded: Promise<WasmModule> | null = null;

export function defaultWasmDir(): string {
  return process.env.CELESTE_GYM_WASM_DIR ?? resolve(REPO_ROOT, "web", "src", "wasm");
}

/** Load (once) the WASM simulator. */
export function loadWasm(dir = defaultWasmDir()): Promise<WasmModule> {
  if (!loaded) {
    loaded = (async () => {
      const glue = resolve(dir, "celeste_wasm.js");
      const binary = resolve(dir, "celeste_wasm_bg.wasm");
      if (!existsSync(glue) || !existsSync(binary)) {
        throw new Error(
          `WASM bundle not found in ${dir}. Build it with \`node scripts/build-wasm.mjs\` ` +
            "from the repository root (needs the wasm32-unknown-unknown target and wasm-bindgen-cli).",
        );
      }
      const module = (await import(pathToFileURL(glue).href)) as WasmModule;
      await module.default({ module_or_path: await readFile(binary) });
      if (typeof module.fuzz_search_msgpack !== "function") {
        throw new Error(
          `WASM bundle in ${dir} is outdated (missing fuzz_search_msgpack); rebuild with node scripts/build-wasm.mjs`,
        );
      }
      return module;
    })();
    loaded.catch(() => {
      loaded = null;
    });
  }
  return loaded;
}

function unpack<T>(bytes: Uint8Array, what: string): T {
  const value = decode(bytes) as T & { success?: boolean; error?: string };
  if (value && typeof value === "object" && !Array.isArray(value) && value.success === false) {
    throw new Error(`${what}: ${value.error ?? "unknown error"}`);
  }
  return value;
}

export async function listRooms(bin: Uint8Array): Promise<string[]> {
  const wasm = await loadWasm();
  return unpack<{ rooms: string[] }>(wasm.list_celeste_map_rooms_msgpack(bin), "list rooms").rooms;
}

export async function auditMap(bin: Uint8Array): Promise<RoomAudit[]> {
  const wasm = await loadWasm();
  return unpack<{ rooms: RoomAudit[] }>(wasm.audit_celeste_map_msgpack(bin), "audit map").rooms;
}

/** Decode one room (`""` = first room) of a `.bin`, or a MessagePack map. */
export async function decodeRoom(bin: Uint8Array, room = ""): Promise<RawMap> {
  const wasm = await loadWasm();
  return unpack<{ map: RawMap }>(wasm.decode_celeste_map_msgpack(bin, room), `decode room ${room || "(first)"}`).map;
}

export function encodeMap(map: RawMap): Uint8Array {
  return encode(map);
}

/** Simulate `inputs.length` frames. Returns `inputs.length + 1` states (index 0 = initial). */
export async function simulateRaw(
  state: SimState,
  inputs: readonly SimInput[],
  mapBytes: Uint8Array,
): Promise<SimState[]> {
  const wasm = await loadWasm();
  const result = unpack<{ states: SimState[] }>(
    wasm.simulate_msgpack(encode(state), encode(inputs), mapBytes, inputs.length),
    "simulate",
  );
  return result.states;
}

export async function fuzzRaw(
  state: SimState,
  mapBytes: Uint8Array,
  spec: FuzzSpec,
  maxCandidates = 0,
): Promise<FuzzRunResult> {
  const wasm = await loadWasm();
  return unpack<FuzzRunResult>(
    wasm.fuzz_search_msgpack(encode(state), mapBytes, JSON.stringify(spec), maxCandidates),
    "fuzz",
  );
}

export async function fuzzResolveInputs(
  spec: FuzzSpec,
  bindings: Record<string, number>,
): Promise<SimInput[]> {
  const wasm = await loadWasm();
  return unpack<SimInput[]>(
    wasm.fuzz_resolve_inputs_msgpack(JSON.stringify(spec), JSON.stringify(bindings)),
    "fuzz resolve inputs",
  );
}

/** Evaluate restricted-Rhai expressions against `current` (all must be true). */
export async function checkState(state: SimState, expressions: readonly string[]): Promise<boolean> {
  const wasm = await loadWasm();
  const value = unpack<boolean>(
    wasm.training_entry_check_msgpack(encode(state), JSON.stringify(expressions)),
    "check",
  );
  return value === true;
}
