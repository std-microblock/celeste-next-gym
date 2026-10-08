/**
 * High-level TypeScript bindings: open a map, simulate, fuzz, render.
 *
 *   import { openMap } from "<repo>/tools/gym-cli/src/index.ts";
 *   const map = await openMap("vanilla:1-ForsakenCity", { room: "1" });
 *   const trace = await map.simulate("20,R;1,R,J;15,R");
 *   await renderTrace(trace, "out.gif", { hud: true });
 */
import { resolve } from "node:path";
import { auditMap, checkState, decodeRoom, encodeMap, fuzzRaw, fuzzResolveInputs, simulateRaw } from "./wasm.ts";
import {
  initialState,
  loadRoom,
  openMapSource,
  rawToView,
  type LoadedRoom,
  type MapSource,
  type OpenMapOptions,
} from "./maps.ts";
import { inputsFromJson, parseTasLines, tasFramesToInputs, describeInput } from "./inputs.ts";
import {
  SceneRenderer,
  contactSheet,
  outputKind,
  writePng,
  writeSequence,
  type RenderOptions,
} from "./render.ts";
import type { FuzzRunResult, FuzzSpec } from "./fuzzTypes.ts";
import type { GymMap, Rect, RoomAudit, SimInput, SimState } from "./types.ts";

export type InputsLike = string | readonly SimInput[] | readonly Record<string, unknown>[];

/** Accept TAS text (`"5,R,J;10,R"`), SimInput[], or FrameButtons[]. */
export async function toInputs(inputs: InputsLike, warnings: string[] = []): Promise<SimInput[]> {
  if (typeof inputs === "string") return tasFramesToInputs(await parseTasLines(inputs), warnings);
  return inputsFromJson(inputs, warnings);
}

export interface StartOptions {
  /** Start position (default: the room's decoded spawn). */
  pos?: { x: number; y: number };
  /** Index into the room's spawn list. */
  spawn?: number;
  /** Fields merged over the default standing state (e.g. `{dashes: 2, facing: false}`). */
  patch?: Partial<SimState>;
}

export class MapSession {
  /** Room audit (every room's bounds and raw entity names), binary maps only. */
  private auditCache?: RoomAudit[];
  private roomViews = new Map<string, GymMap>();

  constructor(
    readonly source: MapSource,
    readonly loaded: LoadedRoom,
  ) {
    this.roomViews.set(loaded.room, loaded.view);
  }

  get room(): string {
    return this.loaded.room;
  }
  get view(): GymMap {
    return this.loaded.view;
  }
  get bounds(): Rect {
    return this.loaded.raw.bounds;
  }
  get spawns(): { x: number; y: number }[] {
    return this.loaded.raw.room_spawns?.length ? this.loaded.raw.room_spawns : [this.loaded.raw.spawn];
  }

  async audit(): Promise<RoomAudit[]> {
    if (!this.source.binary) return [];
    this.auditCache ??= await auditMap(this.source.bytes);
    return this.auditCache;
  }

  /** Default start state, or the timeline's own start state. */
  start(options: StartOptions = {}): SimState {
    const timelineState = this.source.timeline?.initial_state;
    const at = options.pos ?? (options.spawn !== undefined ? this.spawns[options.spawn] : undefined);
    if (options.spawn !== undefined && !at) throw new Error(`spawn index ${options.spawn} out of range (${this.spawns.length} spawns)`);
    const base = timelineState && !at ? { ...timelineState } : initialState(this.loaded, at);
    return { ...base, ...(options.patch ?? {}) } as SimState;
  }

  async simulate(inputs: InputsLike, options: { state?: SimState; warnings?: string[] } = {}): Promise<Trace> {
    const resolved = await toInputs(inputs, options.warnings);
    const state = options.state ?? this.start();
    const states = await simulateRaw(state, resolved, this.loaded.bytes);
    return new Trace(this, resolved, states);
  }

  /**
   * Run the Rust Fuzz searcher from `state` (default: spawn).
   * By default the other rooms' full runtime geometry is dropped (room bounds
   * stay, so transitions are still detected): every prefix-cache node clones
   * the simulator map, and chapter-sized maps make that dominate the search.
   * Pass `withTransitions: true` when candidates must play inside a neighbour room.
   */
  async fuzz(
    spec: FuzzSpec,
    options: { state?: SimState; maxCandidates?: number; withTransitions?: boolean } = {},
  ): Promise<FuzzRunResult> {
    const bytes = options.withTransitions
      ? this.loaded.bytes
      : encodeMap({ ...this.loaded.raw, transition_runtime: [] });
    return fuzzRaw(options.state ?? this.start(), bytes, spec, options.maxCandidates ?? 0);
  }

  /** Replay one fuzz candidate (by bindings) as a full trace. */
  async replayCandidate(spec: FuzzSpec, bindings: Record<string, number>, state?: SimState): Promise<Trace> {
    return this.simulate(await fuzzResolveInputs(spec, bindings), { state });
  }

  /** Room view to render for a state (follows room transitions by `current_room_bounds`). */
  async viewFor(state: SimState): Promise<GymMap> {
    const bounds = state.current_room_bounds as Rect | undefined | null;
    if (!bounds || !this.source.binary) return this.loaded.view;
    const current = this.loaded.view.bounds;
    if (bounds.x === current.x && bounds.y === current.y) return this.loaded.view;
    const room = (await this.audit()).find((audit) => audit.bounds.x === bounds.x && audit.bounds.y === bounds.y);
    if (!room) return this.loaded.view;
    let view = this.roomViews.get(room.name);
    if (!view) {
      view = rawToView(await decodeRoom(this.source.bytes, room.name), this.source.mapPath, room.name);
      this.roomViews.set(room.name, view);
    }
    return view;
  }

  /** Evaluate restricted-Rhai expressions over `current` (all must hold). */
  check(state: SimState, expressions: readonly string[]): Promise<boolean> {
    return checkState(state, expressions);
  }

  /** Re-encode a modified simulator map (e.g. after editing `loaded.raw`). */
  refresh(): void {
    this.loaded.bytes = encodeMap(this.loaded.raw);
  }
}

/** Open a map source and decode one room (see `openMapSource` for accepted specs). */
export async function openMap(spec: string, options: OpenMapOptions & { room?: string } = {}): Promise<MapSession> {
  const source = openMapSource(spec, options);
  return new MapSession(source, await loadRoom(source, options.room));
}

export interface TraceSummary {
  map: string;
  room: string;
  frames: number;
  final: Record<string, unknown>;
  deathFrame: number | null;
  rooms: string[];
  stateCounts: Record<string, number>;
  maxSpeed: { x: number; y: number };
}

export class Trace {
  constructor(
    readonly map: MapSession,
    /** `inputs[i]` produces `states[i + 1]`. */
    readonly inputs: SimInput[],
    /** `states[0]` is the start state. */
    readonly states: SimState[],
  ) {}

  get final(): SimState {
    return this.states[this.states.length - 1];
  }

  async summary(): Promise<TraceSummary> {
    const stateCounts: Record<string, number> = {};
    let deathFrame: number | null = null;
    let maxX = 0;
    let maxY = 0;
    const rooms: string[] = [];
    let lastBounds = "";
    for (const [index, state] of this.states.entries()) {
      stateCounts[state.state] = (stateCounts[state.state] ?? 0) + 1;
      if (state.dead && deathFrame === null) deathFrame = index;
      maxX = Math.max(maxX, Math.abs(state.speed.x));
      maxY = Math.max(maxY, Math.abs(state.speed.y));
      const bounds = JSON.stringify(state.current_room_bounds ?? null);
      if (bounds !== lastBounds) {
        lastBounds = bounds;
        const view = await this.map.viewFor(state);
        if (rooms.at(-1) !== view.room) rooms.push(view.room ?? "");
      }
    }
    return {
      map: this.map.source.mapPath,
      room: this.map.room,
      frames: this.inputs.length,
      final: compactState(this.final),
      deathFrame,
      rooms,
      stateCounts,
      maxSpeed: { x: maxX, y: maxY },
    };
  }

  /** One text row per frame: `frame state pos speed dashes stamina flags input`. */
  table(options: { every?: number; from?: number; to?: number } = {}): string {
    const every = Math.max(1, options.every ?? 1);
    const from = Math.max(0, options.from ?? 0);
    const to = Math.min(this.states.length - 1, options.to ?? this.states.length - 1);
    const rows = ["frame state           pos.x     pos.y    spd.x    spd.y  dash stamina flags  input"];
    for (let i = from; i <= to; i += every) {
      const s = this.states[i];
      const flags = `${s.on_ground ? "G" : "-"}${s.ducking ? "D" : "-"}${s.dead ? "X" : "-"}${s.facing ? ">" : "<"}`;
      rows.push(
        `${String(i).padStart(5)} ${s.state.padEnd(14)} ${s.pos.x.toFixed(2).padStart(9)} ${s.pos.y.toFixed(2).padStart(9)} ` +
          `${s.speed.x.toFixed(2).padStart(8)} ${s.speed.y.toFixed(2).padStart(8)} ${String(s.dashes).padStart(5)} ` +
          `${s.stamina.toFixed(1).padStart(7)} ${flags}   ${i > 0 ? describeInput(this.inputs[i - 1]) : ""}`,
      );
    }
    return rows.join("\n");
  }

  toJSON(options: { full?: boolean } = {}) {
    return {
      map: this.map.source.mapPath,
      room: this.map.room,
      inputs: this.inputs,
      states: options.full ? this.states : this.states.map(compactState),
    };
  }
}

/** The fields that matter for comparisons (AGENTS.md E2E field set). */
export function compactState(state: SimState): Record<string, unknown> {
  return {
    pos: state.pos,
    speed: state.speed,
    state: state.state,
    facing: state.facing,
    dashes: state.dashes,
    stamina: state.stamina,
    on_ground: state.on_ground,
    ducking: state.ducking,
    dead: state.dead,
  };
}

export interface TraceRenderOptions extends RenderOptions {
  /** Render every Nth frame (default 1; GIF default 2). */
  every?: number;
  from?: number;
  to?: number;
  /** PNG only: which frame (default last). */
  frame?: number;
  /** PNG only: contact sheet with this many thumbnails (or use `sheetEvery`). */
  sheet?: number;
  sheetEvery?: number;
  sheetColumns?: number;
  fps?: number;
  ffmpeg?: string;
}

/** Render a trace to `.png` (frame or contact sheet), `.gif`, `.mp4/.webm/.mkv/.mov`, or a PNG directory. */
export async function renderTrace(trace: Trace, out: string, options: TraceRenderOptions = {}) {
  const kind = outputKind(out);
  const renderer = await SceneRenderer.create(trace.map.source.mapPath, options);
  const last = trace.states.length - 1;
  const from = Math.max(0, Math.min(last, options.from ?? 0));
  const to = Math.max(from, Math.min(last, options.to ?? last));
  const views: GymMap[] = [];
  for (const state of trace.states) views.push(await trace.map.viewFor(state));

  if (kind === "png" && (options.sheet || options.sheetEvery)) {
    const indices: number[] = [];
    if (options.sheetEvery) for (let i = from; i <= to; i += options.sheetEvery) indices.push(i);
    else {
      const count = Math.max(1, Math.min(options.sheet ?? 12, to - from + 1));
      for (let k = 0; k < count; k += 1) indices.push(Math.round(from + ((to - from) * k) / Math.max(1, count - 1)));
    }
    if (indices.at(-1) !== to && options.sheetEvery) indices.push(to);
    const frames = [...new Set(indices)].map((index) => ({
      canvas: renderer.drawFrame(views[index], trace.states, index, trace.inputs, undefined, renderer.size(trace.map.view)),
      label: `f${index} ${trace.states[index].state}${trace.states[index].dead ? " DEAD" : ""}`,
    }));
    const sheet = contactSheet(frames, options.sheetColumns ?? Math.min(4, frames.length));
    writePng(sheet, out);
    return { kind: "sheet" as const, path: resolve(out), frames: frames.length, width: sheet.width, height: sheet.height };
  }
  if (kind === "png") {
    const index = Math.max(0, Math.min(last, options.frame ?? to));
    const canvas = renderer.drawFrame(views[index], trace.states, index, trace.inputs);
    writePng(canvas, out);
    return { kind, path: resolve(out), frames: 1, frame: index, width: canvas.width, height: canvas.height };
  }
  const every = Math.max(1, options.every ?? (kind === "gif" ? 2 : 1));
  const indices: number[] = [];
  for (let i = from; i <= to; i += every) indices.push(i);
  if (indices.at(-1) !== to) indices.push(to);
  // Keep one output size for the whole sequence (the start room's size).
  const size = renderer.size(trace.map.view);
  const fps = options.fps ?? Math.round(60 / every);
  const result = await writeSequence(
    {
      width: size.width,
      height: size.height,
      count: indices.length,
      frame: (i) => renderer.drawFrame(views[indices[i]], trace.states, indices[i], trace.inputs, undefined, size),
    },
    out,
    { fps, ffmpeg: options.ffmpeg },
  );
  return { ...result, fps, width: size.width, height: size.height, every };
}

/** Render a single state (no simulation) to a PNG. */
export async function renderState(map: MapSession, state: SimState, out: string, options: RenderOptions = {}) {
  return renderTrace(new Trace(map, [], [state]), out, options);
}
