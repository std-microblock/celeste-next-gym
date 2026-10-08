/**
 * celeste-gym — agent-oriented CLI over the Celeste Next Gym WASM simulator.
 * Run `celeste-gym help` for usage. All commands print one JSON document on
 * stdout (human tables, when requested, go to stderr) and exit non-zero on failure.
 */
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, extname, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import * as api from "./index.ts";
import { openMap, renderState, renderTrace, Trace, type MapSession, type TraceRenderOptions } from "./gym.ts";
import { listZipMaps, openMapSource, findVanillaMap } from "./maps.ts";
import { checkMap } from "./check.ts";
import { inputsFromFile, inputsToTas, parseTasLines, resolveTasFile, tasFramesToInputs } from "./inputs.ts";
import { listThemes } from "./render.ts";
import { auditMap, defaultWasmDir, fuzzResolveInputs, listRooms, loadWasm } from "./wasm.ts";
import { REPO_ROOT, VANILLA_MAPS, WEB_PUBLIC } from "./paths.ts";
import type { FuzzRunResult, FuzzSpec } from "./fuzzTypes.ts";
import type { SimInput, SimState } from "./types.ts";

const HELP = `celeste-gym <command> [options]

Map sources (<map>):
  playground                    bundled Mechanics Playground
  vanilla:<name>                repo-owned vanilla map, e.g. vanilla:1-ForsakenCity, vanilla:1H, vanilla:7
  path/to/map.bin               Celeste BinaryPacker map
  path/to/Mod.zip [--bin P]     Everest mod zip; --bin picks Maps/**.bin (path or unique substring)
  path/to/map.json              simulator map JSON, or timeline v2 {version:2,map,initial_state,inputs}

Commands:
  info                          WASM/asset/ffmpeg status
  themes [--filter S]           list render themes
  maps <map|zip>                list maps inside a zip, or rooms of a map (bounds, spawns, entities)
  check <map|zip> [--room R] [--all-maps] [--frames N] [--strict]
                                verify a map loads: decode every room, report unsupported
                                entities, run N idle frames (default 30). Exit 1 on error
                                (or on partial with --strict)
  sim <map> [--room R] (--input "5,R;1,R,J" | --inputs FILE) [start] [--frames N] [output] [render]
                                simulate inputs; FILE = .json (FrameButtons/SimInput/timeline),
                                TAS action lines, or .tas
  tas <file.tas> [--map M] [--room R] [--from-label L] [--to-label L] [--start-frame N] [--frames N]
                                flatten a CelesteTAS file (Read/Repeat/labels) and simulate it.
                                Map/room default to the TAS 'console load' / lvl_ label.
  fuzz <spec.json|spec.ts> [--map M] [--room R] [start] [--max-candidates N] [--top N]
       [--bindings JSON] [--candidate JSON] [--with-transitions] [render]
                                run the Rust Fuzz searcher; .ts specs may export default spec,
                                a (ctx)=>spec function, and optional map/room/state/bin exports
  render <map> [--room R] [--state FILE | --trace FILE] --render OUT [render]
                                render the room (spawn), a saved state, or a saved trace
  run <script.ts> [-- args...]  run a TypeScript script; its default export receives (api, args)

Start state [start]:
  --state FILE                  JSON snapshot (full SimState); a trace file uses its last state
  --pos X,Y | --spawn I         start position / room spawn index
  --patch JSON                  merge fields, e.g. '{"dashes":2,"facing":false}'

Output:
  --out FILE                    write trace / result JSON (sim/tas: {map,room,inputs,states})
  --full                        keep every snapshot field in --out (default: compact fields)
  --table                       print a per-frame table to stderr (--every N, --from, --to)
  --tas-out FILE                write the inputs back as TAS lines
  --frames N                    sim: pad with idle frames to N; tas: limit to N frames

Render [render] (repeat --render for several outputs):
  --render OUT                  .png (frame or contact sheet), .gif, .mp4/.webm/.mkv/.mov, or a
                                directory (PNG sequence). Video needs ffmpeg (PATH, FFMPEG or --ffmpeg).
  --scale N (2) --camera follow|room --theme ID --hud --hitboxes --path
  --every N --fps N --from F --to F --frame F (png) --sheet N | --sheet-every N --columns C
`;

const OPTIONS = {
  help: { type: "boolean", short: "h" },
  room: { type: "string" },
  bin: { type: "string" },
  map: { type: "string" },
  input: { type: "string", multiple: true },
  inputs: { type: "string" },
  state: { type: "string" },
  trace: { type: "string" },
  pos: { type: "string" },
  spawn: { type: "string" },
  patch: { type: "string" },
  frames: { type: "string" },
  out: { type: "string" },
  "tas-out": { type: "string" },
  full: { type: "boolean" },
  table: { type: "boolean" },
  every: { type: "string" },
  from: { type: "string" },
  to: { type: "string" },
  frame: { type: "string" },
  render: { type: "string", multiple: true },
  scale: { type: "string" },
  camera: { type: "string" },
  theme: { type: "string" },
  hud: { type: "boolean" },
  hitboxes: { type: "boolean" },
  path: { type: "boolean" },
  fps: { type: "string" },
  sheet: { type: "string" },
  "sheet-every": { type: "string" },
  columns: { type: "string" },
  ffmpeg: { type: "string" },
  filter: { type: "string" },
  "all-maps": { type: "boolean" },
  strict: { type: "boolean" },
  rooms: { type: "boolean" },
  "from-label": { type: "string" },
  "to-label": { type: "string" },
  "start-frame": { type: "string" },
  "tas-root": { type: "string" },
  "max-candidates": { type: "string" },
  top: { type: "string" },
  candidate: { type: "string" },
  bindings: { type: "string" },
  "with-transitions": { type: "boolean" },
  quiet: { type: "boolean", short: "q" },
} as const;

type Values = ReturnType<typeof parse>["values"];

function parse(argv: string[]) {
  return parseArgs({ args: argv, options: OPTIONS, allowPositionals: true, strict: true });
}

function int(value: string | undefined, name: string): number | undefined {
  if (value === undefined) return undefined;
  const parsed = Number(value);
  if (!Number.isInteger(parsed)) throw new Error(`--${name} must be an integer, got "${value}"`);
  return parsed;
}

function print(value: unknown): void {
  process.stdout.write(JSON.stringify(value, null, 2) + "\n");
}

function writeJson(path: string, value: unknown): string {
  const full = resolve(path);
  mkdirSync(dirname(full), { recursive: true });
  writeFileSync(full, JSON.stringify(value, null, 2) + "\n");
  return full;
}

function readJson(path: string): unknown {
  return JSON.parse(readFileSync(path, "utf8"));
}

function renderOptions(values: Values): TraceRenderOptions {
  const camera = values.camera;
  if (camera && camera !== "follow" && camera !== "room") throw new Error("--camera must be follow or room");
  return {
    scale: int(values.scale, "scale"),
    camera: camera as "follow" | "room" | undefined,
    theme: values.theme,
    hud: values.hud,
    hitboxes: values.hitboxes,
    path: values.path,
    every: int(values.every, "every"),
    fps: int(values.fps, "fps"),
    from: int(values.from, "from"),
    to: int(values.to, "to"),
    frame: int(values.frame, "frame"),
    sheet: int(values.sheet, "sheet"),
    sheetEvery: int(values["sheet-every"], "sheet-every"),
    sheetColumns: int(values.columns, "columns"),
    ffmpeg: values.ffmpeg,
  };
}

function startState(map: MapSession, values: Values): SimState {
  let base: SimState | undefined;
  if (values.state) {
    const document = readJson(values.state) as Record<string, unknown>;
    if (Array.isArray(document.states)) base = document.states.at(-1) as SimState;
    else if (document.initial_state) base = document.initial_state as SimState;
    else if (document.final_state) base = document.final_state as SimState;
    else base = document as unknown as SimState;
    if (!base?.pos) throw new Error(`${values.state}: no state found (expected a SimState, trace, or timeline)`);
  }
  let pos: { x: number; y: number } | undefined;
  if (values.pos) {
    const [x, y] = values.pos.split(",").map(Number);
    if (!Number.isFinite(x) || !Number.isFinite(y)) throw new Error("--pos must be X,Y");
    pos = { x, y };
  }
  const patch = values.patch ? (JSON.parse(values.patch) as Partial<SimState>) : {};
  if (base) return { ...base, ...(pos ? { pos } : {}), ...patch } as SimState;
  return map.start({ pos, spawn: int(values.spawn, "spawn"), patch });
}

async function renderOutputs(trace: Trace, values: Values) {
  const outputs = [];
  for (const out of values.render ?? []) outputs.push(await renderTrace(trace, out, renderOptions(values)));
  return outputs;
}

async function finishTrace(trace: Trace, values: Values, extra: Record<string, unknown>, warnings: string[]) {
  if (values.table) {
    process.stderr.write(
      trace.table({ every: int(values.every, "every"), from: int(values.from, "from"), to: int(values.to, "to") }) + "\n",
    );
  }
  const written: Record<string, string> = {};
  if (values.out) written.trace = writeJson(values.out, trace.toJSON({ full: values.full }));
  if (values["tas-out"]) {
    const path = resolve(values["tas-out"]);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, inputsToTas(trace.inputs) + "\n");
    written.tas = path;
  }
  const renders = await renderOutputs(trace, values);
  print({
    ok: true,
    ...extra,
    summary: await trace.summary(),
    ...(Object.keys(written).length ? { written } : {}),
    ...(renders.length ? { renders } : {}),
    ...(warnings.length ? { warnings: [...new Set(warnings)] } : {}),
  });
}

function padIdle(inputs: SimInput[], frames: number | undefined): SimInput[] {
  if (frames === undefined || frames <= inputs.length) return frames === undefined ? inputs : inputs.slice(0, frames);
  const idle: SimInput = {
    move_x: 0,
    move_y: 0,
    jump_pressed: false,
    jump_held: false,
    dash_pressed: false,
    crouch_dash_pressed: false,
    grab_held: false,
    talk_pressed: false,
  };
  return [...inputs, ...Array.from({ length: frames - inputs.length }, () => ({ ...idle }))];
}

// ---------------------------------------------------------------------------

async function cmdInfo() {
  let wasm: Record<string, unknown>;
  try {
    await loadWasm();
    wasm = { ok: true, dir: defaultWasmDir() };
  } catch (error) {
    wasm = { ok: false, dir: defaultWasmDir(), error: (error as Error).message };
  }
  let ffmpeg: string | null = null;
  try {
    ffmpeg = (await import("./render.ts")).findFfmpeg();
  } catch {
    ffmpeg = null;
  }
  print({
    repo: REPO_ROOT,
    wasm,
    assets: { dir: WEB_PUBLIC, ok: existsSync(resolve(WEB_PUBLIC, "assets/original/gameplay/gameplay-selected.png")) },
    vanillaMaps: { dir: VANILLA_MAPS, ok: existsSync(VANILLA_MAPS) },
    ffmpeg,
    node: process.version,
  });
}

async function cmdThemes(values: Values) {
  const filter = values.filter?.toLowerCase();
  print(
    listThemes().filter(
      (theme) => !filter || `${theme.id} ${theme.label} ${theme.chapter}`.toLowerCase().includes(filter),
    ),
  );
}

async function cmdMaps(positionals: string[], values: Values) {
  const spec = positionals[0];
  if (!spec) throw new Error("usage: celeste-gym maps <map|zip>");
  if (extname(spec).toLowerCase() === ".zip" && !values.bin) {
    print({ zip: resolve(spec), maps: listZipMaps(spec) });
    return;
  }
  const source = openMapSource(spec, { bin: values.bin });
  if (!source.binary) {
    const map = await openMap(spec, { bin: values.bin });
    print({ map: source.mapPath, rooms: [{ name: "", bounds: map.bounds, spawns: map.spawns.length }] });
    return;
  }
  const audit = await auditMap(source.bytes);
  print({
    map: source.mapPath,
    source: source.entry ? `${source.path}!${source.entry}` : source.path,
    rooms: audit.map((room) => ({
      name: room.name,
      bounds: room.bounds,
      spawns: room.spawns.length,
      entities: Object.values(room.entity_names).reduce((sum, count) => sum + count, 0),
      triggers: Object.values(room.trigger_names).reduce((sum, count) => sum + count, 0),
    })),
  });
}

async function cmdCheck(positionals: string[], values: Values) {
  const spec = positionals[0];
  if (!spec) throw new Error("usage: celeste-gym check <map|zip>");
  const frames = int(values.frames, "frames");
  const isZip = extname(spec).toLowerCase() === ".zip";
  const targets =
    isZip && values["all-maps"]
      ? listZipMaps(spec).map((map) => openMapSource(spec, { bin: map.entry }))
      : [openMapSource(spec, { bin: values.bin })];
  const reports: Awaited<ReturnType<typeof checkMap>>[] = [];
  for (const source of targets) reports.push(await checkMap(source, { room: values.room, frames }));
  const status = reports.some((report) => report.status === "error")
    ? "error"
    : reports.some((report) => report.status === "partial")
      ? "partial"
      : "ok";
  const compact = (report: (typeof reports)[number]) => ({
    ...report,
    // Keep stdout readable: room details only for problem rooms unless a single room was asked for.
    results: values.room ? report.results : report.results.filter((room) => room.status !== "ok"),
    okRooms: values.room ? undefined : report.results.filter((room) => room.status === "ok").map((room) => room.room),
  });
  print(reports.length === 1 ? compact(reports[0]) : { status, maps: reports.map(compact) });
  if (status === "error" || (values.strict && status === "partial")) process.exitCode = 1;
}

async function cmdSim(positionals: string[], values: Values) {
  const spec = positionals[0] ?? values.map;
  if (!spec) throw new Error("usage: celeste-gym sim <map> --input ... | --inputs FILE");
  const map = await openMap(spec, { bin: values.bin, room: values.room });
  const warnings: string[] = [];
  let inputs: SimInput[];
  if (values.inputs) inputs = await inputsFromFile(values.inputs, warnings);
  else if (values.input?.length) inputs = tasFramesToInputs(await parseTasLines(values.input.join(";")), warnings);
  else if (map.source.timeline?.inputs) inputs = await api.inputsFromJson(map.source.timeline.inputs, warnings);
  else inputs = [];
  inputs = padIdle(inputs, int(values.frames, "frames"));
  if (inputs.length === 0) throw new Error("no inputs: pass --input, --inputs or --frames N (idle)");
  const trace = await map.simulate(inputs, { state: startState(map, values), warnings });
  await finishTrace(trace, values, { command: "sim" }, warnings);
}

const VANILLA_MODE_SUFFIX = { A: "", B: "H", C: "X" } as const;

async function cmdTas(positionals: string[], values: Values) {
  const file = positionals[0];
  if (!file) throw new Error("usage: celeste-gym tas <file.tas> [--map M]");
  const warnings: string[] = [];
  const tas = await resolveTasFile(
    file,
    {
      root: values["tas-root"],
      fromLabel: values["from-label"],
      toLabel: values["to-label"],
      startFrame: int(values["start-frame"], "start-frame"),
      frameCount: int(values.frames, "frames"),
    },
    warnings,
  );
  let mapSpec = values.map;
  if (!mapSpec && tas.consoleLoad && existsSync(VANILLA_MAPS)) {
    const area = tas.consoleLoad.area.replace(/^Celeste\//, "");
    mapSpec = /^\d+$/.test(area)
      ? `vanilla:${area}${VANILLA_MODE_SUFFIX[tas.consoleLoad.mode]}`
      : `vanilla:${area}`;
    findVanillaMap(mapSpec.slice(8));
  }
  if (!mapSpec) throw new Error("no map: pass --map (the TAS has no usable 'console load' command)");
  const labelRoom = values["from-label"]?.replace(/^#/, "").match(/^lvl_(.+)$/)?.[1];
  const room = values.room ?? labelRoom ?? (tas.startFrame === 0 ? tas.consoleLoad?.room : undefined);
  const map = await openMap(mapSpec, { bin: values.bin, room });
  let state: SimState;
  if (values.state || values.pos || values.spawn || values.patch) state = startState(map, values);
  else if (tas.startFrame === 0 && tas.consoleLoad?.position && !room) state = map.start({ pos: tas.consoleLoad.position });
  else state = map.start();
  if (tas.inputs.length === 0) throw new Error("TAS selection contains no frames");
  warnings.push(
    "next-gym starts from a standing snapshot; real-game intro/cutscene/transition frames are not simulated, so long TAS segments are expected to diverge (use --from-label / --state to anchor)",
  );
  let trace: Trace;
  try {
    trace = await map.simulate(tas.inputs, { state, warnings });
  } catch (error) {
    throw new Error(`${(error as Error).message} (TAS frames ${tas.startFrame}-${tas.endFrame})`);
  }
  await finishTrace(
    trace,
    values,
    {
      command: "tas",
      tas: {
        file: tas.file,
        totalFrames: tas.totalFrames,
        startFrame: tas.startFrame,
        endFrame: tas.endFrame,
        consoleLoad: tas.consoleLoad,
        labels: tas.labels.slice(0, 200),
      },
      map: mapSpec,
      room: map.room,
    },
    warnings,
  );
}

interface FuzzModule {
  spec: FuzzSpec;
  map?: string;
  room?: string;
  bin?: string;
  state?: Partial<SimState>;
}

async function loadFuzzModule(file: string, mapFor: (spec: string, room?: string, bin?: string) => Promise<MapSession>): Promise<FuzzModule & { session?: MapSession }> {
  const extension = extname(file).toLowerCase();
  if (extension === ".json") {
    const document = readJson(file) as Record<string, unknown>;
    // Allow {map, room, state, spec} wrappers as well as a bare spec.
    if (document.spec && typeof document.spec === "object") return document as unknown as FuzzModule;
    return { spec: document as unknown as FuzzSpec };
  }
  const module = (await import(pathToFileURL(resolve(file)).href)) as Record<string, unknown>;
  const exported = module.default ?? module.fuzz ?? module.spec;
  const meta = {
    map: module.map as string | undefined,
    room: module.room as string | undefined,
    bin: module.bin as string | undefined,
    state: module.state as Partial<SimState> | undefined,
  };
  if (typeof exported === "function") {
    const session = meta.map ? await mapFor(meta.map, meta.room, meta.bin) : undefined;
    const spec = (await (exported as (ctx: unknown) => unknown)({ api, map: session })) as FuzzSpec;
    return { ...meta, spec, session };
  }
  if (!exported || typeof exported !== "object") throw new Error(`${file}: export default a FuzzSpec or a function returning one`);
  return { ...meta, spec: exported as FuzzSpec };
}

async function cmdFuzz(positionals: string[], values: Values) {
  const file = positionals[0];
  if (!file) throw new Error("usage: celeste-gym fuzz <spec.json|spec.ts> --map M");
  const mapFor = (spec: string, room?: string, bin?: string) =>
    openMap(values.map ?? spec, { room: values.room ?? room, bin: values.bin ?? bin });
  const loaded = await loadFuzzModule(file, mapFor);
  const mapSpec = values.map ?? loaded.map;
  if (!mapSpec) throw new Error("no map: pass --map or export `map` from the spec module");
  const map = loaded.session ?? (await openMap(mapSpec, { bin: values.bin ?? loaded.bin, room: values.room ?? loaded.room }));
  let state = startState(map, values);
  if (loaded.state && !values.state) state = { ...state, ...loaded.state } as SimState;
  const spec: FuzzSpec = { ...loaded.spec };
  const top = int(values.top, "top");
  if (top) spec.search = { ...(spec.search ?? {}), output: [...new Set([...(spec.search?.output ?? ["best", "windows"]), `top_${top}` as const])] };
  if (values.bindings) spec.search = { ...(spec.search ?? {}), bindings: JSON.parse(values.bindings) };
  const started = Date.now();
  const run: FuzzRunResult = await map.fuzz(spec, {
    state,
    maxCandidates: int(values["max-candidates"], "max-candidates"),
    withTransitions: values["with-transitions"],
  });
  const elapsedMs = Date.now() - started;

  // Replay the best (or a chosen) candidate for tables / renders / --out traces.
  let replay: Trace | undefined;
  let replayBindings = run.result.best?.bindings;
  if (values.candidate) replayBindings = JSON.parse(values.candidate);
  if (replayBindings) {
    const inputs = values.candidate ? await fuzzResolveInputs(spec, replayBindings) : run.best_inputs ?? (await fuzzResolveInputs(spec, replayBindings));
    replay = await map.simulate(inputs, { state });
  }
  const written: Record<string, string> = {};
  if (values.out) written.result = writeJson(values.out, { spec, map: mapSpec, room: map.room, ...run });
  if (replay && values["tas-out"]) {
    const path = resolve(values["tas-out"]);
    writeFileSync(path, inputsToTas(replay.inputs) + "\n");
    written.tas = path;
  }
  if (replay && values.table) process.stderr.write(replay.table({ every: int(values.every, "every") }) + "\n");
  const renders = replay ? await renderOutputs(replay, values) : [];
  const brief = (candidate: FuzzRunResult["result"]["best"]) =>
    candidate && {
      bindings: candidate.bindings,
      objective_values: candidate.objective_values,
      verified_inputs: candidate.verified_inputs,
      final: api.compactState(candidate.final_state as unknown as SimState),
    };
  print({
    ok: true,
    command: "fuzz",
    map: map.source.mapPath,
    room: map.room,
    elapsedMs,
    estimatedCandidates: run.estimated_candidates,
    stats: run.result.stats,
    best: brief(run.result.best),
    top: run.result.top.map(brief),
    exactWindows: run.result.exact_windows,
    connectedRegions: run.result.connected_regions.map((region) => ({
      bounds: region.bounds,
      successful_count: region.successful_count,
      density: region.density,
      best: region.best.bindings,
    })),
    coverage: run.result.coverage_report ?? undefined,
    ...(replay ? { replay: { bindings: replayBindings, summary: await replay.summary(), tas: inputsToTas(replay.inputs) } } : {}),
    ...(Object.keys(written).length ? { written } : {}),
    ...(renders.length ? { renders } : {}),
  });
  if (!run.result.best) process.exitCode = 2;
}

async function cmdRender(positionals: string[], values: Values) {
  const spec = positionals[0] ?? values.map;
  if (!spec) throw new Error("usage: celeste-gym render <map> --render OUT");
  if (!values.render?.length) throw new Error("pass --render OUT (.png/.gif/.mp4/...)");
  const map = await openMap(spec, { bin: values.bin, room: values.room });
  const renders = [];
  if (values.trace) {
    const document = readJson(values.trace) as { inputs?: SimInput[]; states?: SimState[] };
    if (!document.states?.length) throw new Error(`${values.trace}: expected a trace {inputs, states}`);
    const trace = new Trace(map, document.inputs ?? [], document.states);
    for (const out of values.render) renders.push(await renderTrace(trace, out, renderOptions(values)));
  } else {
    const state = startState(map, values);
    for (const out of values.render) renders.push(await renderState(map, state, out, renderOptions(values)));
  }
  print({ ok: true, command: "render", map: map.source.mapPath, room: map.room, renders });
}

async function cmdRun(positionals: string[], rest: string[]) {
  const file = positionals[0];
  if (!file) throw new Error("usage: celeste-gym run <script.ts> [-- args...]");
  const module = (await import(pathToFileURL(resolve(file)).href)) as { default?: unknown };
  if (typeof module.default === "function") {
    const result = await (module.default as (gym: typeof api, args: string[]) => unknown)(api, rest);
    if (result !== undefined) print(result);
  }
}

export async function main(argv: string[]): Promise<void> {
  const dashDash = argv.indexOf("--");
  const own = dashDash >= 0 ? argv.slice(0, dashDash) : argv;
  const rest = dashDash >= 0 ? argv.slice(dashDash + 1) : [];
  const [command, ...tail] = own;
  if (!command || command === "help" || command === "--help" || command === "-h") {
    process.stdout.write(HELP);
    return;
  }
  if (command === "run") {
    // Scripts own their arguments: everything after the file is passed through.
    await cmdRun(tail.slice(0, 1), [...tail.slice(1), ...rest]);
    return;
  }
  const { values, positionals } = parse(tail);
  if (values.help) {
    process.stdout.write(HELP);
    return;
  }
  switch (command) {
    case "info":
      return cmdInfo();
    case "themes":
      return cmdThemes(values);
    case "maps":
    case "rooms":
      return cmdMaps(positionals, values);
    case "check":
      return cmdCheck(positionals, values);
    case "sim":
      return cmdSim(positionals, values);
    case "tas":
      return cmdTas(positionals, values);
    case "fuzz":
      return cmdFuzz(positionals, values);
    case "render":
      return cmdRender(positionals, values);
    default:
      throw new Error(`unknown command "${command}" (see celeste-gym help)`);
  }
}

// Keep unused-import linting quiet for helpers re-exported to scripts.
void listRooms;
