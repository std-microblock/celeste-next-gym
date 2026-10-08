---
name: celeste-gym-cli
description: Drive the Celeste Next Gym Rust/WASM simulator from the command line or TypeScript — simulate input sequences, replay CelesteTAS .tas files, run the Fuzz input-space searcher (JSON or .ts specs), check whether a .bin / Everest mod .zip map loads, and render results to PNG frames, contact sheets, GIF or MP4/WebM. Use for any celeste-next-gym task that needs to run, search, verify or *see* player physics on a map (e.g. "does this hyper work here", "find the frame window", "render this TAS", "can this mod map load", "make a video of the trace").
---

# celeste-gym CLI

Agent-oriented CLI in `tools/gym-cli` (repo `celeste-next-gym`). It runs the same
`celeste-physics` WASM bundle as the web app, and renders with the web app's own canvas
renderer (`web/src/render/gameRenderer.ts`) on `@napi-rs/canvas`.

Every command prints **one JSON document on stdout** (`{"ok":false,"error":...}` + exit 1 on
failure). Human tables (`--table`) go to stderr. Set `CELESTE_GYM_DEBUG=1` for stack traces.

## Setup (once per checkout)

```powershell
cd tools/gym-cli; npm install            # @napi-rs/canvas, tsx, gifenc, fflate, msgpack
node scripts/build-wasm.mjs              # from repo root, only if `info` says wasm missing/outdated
node tools/gym-cli/bin/celeste-gym.mjs info   # wasm / assets / vanilla maps / ffmpeg status
```

Alias used below: `cg` = `node tools/gym-cli/bin/celeste-gym.mjs` (run from repo root).
Rebuild WASM after changing anything under `crates/` — the CLI uses `web/src/wasm/`.
`CELESTE_GYM_WASM_DIR` overrides the bundle directory.

## Map sources (`<map>`)

| spec | meaning |
| --- | --- |
| `playground` | bundled Mechanics Playground (rooms `playground`, `transition_0`) |
| `vanilla:1-ForsakenCity`, `vanilla:1`, `vanilla:1H`, `vanilla:7X` | `vendor/celeste-game/Content/Maps` (A/B=H/C=X sides) |
| `path/map.bin` | Celeste BinaryPacker map |
| `path/Mod.zip --bin <path-or-substring>` | Everest mod zip; `--bin` picks `Maps/**.bin` when it has several |
| `path/map.json` | simulator Map JSON, or timeline v2 `{version:2,map,initial_state,inputs}` (start state + inputs are used automatically) |

`--room 1` or `--room lvl_1` selects the room (default: first room). List rooms/maps with
`cg maps <map|zip>`.

## Commands

```text
cg maps <map|zip>                       rooms (bounds, spawns, entity/trigger counts) or maps in a zip
cg check <map|zip> [--room R] [--all-maps] [--frames N] [--strict]
cg sim <map> [--room R] (--input "..." | --inputs FILE) [start] [--frames N] [out] [render]
cg tas <file.tas> [--map M] [--room R] [--from-label lvl_2] [--to-label lvl_3] [--start-frame N] [--frames N]
cg fuzz <spec.json|spec.ts> [--map M] [--room R] [start] [--top N] [--bindings JSON] [--candidate JSON]
        [--max-candidates N] [--with-transitions] [--out result.json] [render]
cg render <map> [--room R] [--state FILE | --trace FILE] --render OUT [render]
cg run <script.ts> [args...]            default export (api, args) => result (printed as JSON)
cg themes [--filter S]
```

Start state `[start]`: default = standing at the room's decoded spawn, `dashes:1`,
`stamina:110`, facing right. `--spawn I` (room spawn index), `--pos X,Y`,
`--patch '{"dashes":2,"facing":false}'`, `--state FILE` (a SimState, a trace's last state, a
fuzz result's `final_state`, or a timeline's `initial_state`). Positions are Celeste world
pixels; `pos` is the player's bottom-centre.

Outputs `[out]`: `--out trace.json` (`{map,room,inputs,states}`; compact fields unless
`--full`), `--table` (per-frame `frame state pos speed dash stamina flags input` on stderr;
`--every/--from/--to`), `--tas-out file.tas` (inputs back as TAS lines). Frame `i` of a trace
is the state *after* `inputs[i-1]`; frame 0 is the start state.

### check (is this map loadable?)

Decodes every room with the simulator's decoder, runs `--frames` (default 30) idle frames from
the spawn, and reports per room `ok | partial | error`:
- `partial` = loads and simulates, but has raw entities decoded as `unknown`
  (`unsupportedEntities`, e.g. `crumbleBlock`, `dashBlock`) — those objects are ignored by
  physics (and drawn as grey placeholder boxes).
- `error` = decode or simulation failure (message in `error` / `simulation.error`).
Exit code 1 on any error (also on partial with `--strict`). Only non-ok rooms are listed in
`results`; `okRooms` names the rest. `--room R` gives full detail for one room.

## Input syntax

CelesteTAS action lines: `frames,keys...`. Inline `--input` separates lines with `;` (or repeat
`--input`). Keys: `L R U D` move, `J`/`K` jump (two bindings — `K` while `J` is held is a new
press), `X`/`C` dash, `Z`/`V` crouch (demo) dash, `G`/`H` grab (held), `A<dirs>` dash-only aim,
`M<dirs>` move-only, `F,<angle>[,<mag>]` analog (quantised to 8 directions). A button held over
consecutive lines is one press; to re-press, leave a frame without it or alternate `J`/`K`.
Opposite directions held together follow Celeste's TakeNewer axis (newer wins).

```text
"10;1,R,X;20,R"            idle 10, right-dash, hold right 20
"5,R;1,R,J;11,R,J;10,R"    run, full-height jump (hold J 12 frames = max var jump)
"1,R,D,X;3,R;1,R,J"        hyper/wavedash shape
```

`--inputs FILE`: `.json` array of FrameButtons `{left,right,up,down,jump,dash,crouch_dash,grab}`,
of SimInput `{move_x,move_y,jump_pressed,jump_held,dash_pressed,crouch_dash_pressed,grab_held}`,
of TAS strings, a `{inputs:[...]}` document; or a text file of TAS lines; or a `.tas` file.
`--frames N` pads with idle frames (or truncates).

## TAS replay

`cg tas 1A.tas --from-label lvl_3 --to-label lvl_4 --render out.mp4 --hud`
- `.tas` trees are flattened by `tools/tas-replay` (Read/Repeat/labels/StunPause...); `Read`
  resolves relative to the file (or `--tas-root`).
- Map defaults to the TAS's `console load <area>` (vanilla only); room defaults to
  `lvl_<room>` from `--from-label`. Output lists all `labels` with their frame numbers.
- The simulator starts from a *standing snapshot*: intro/cutscene/transition frames and the
  real arrival state are not reproduced, so TAS segments drift or die after a while. To replay
  faithfully, anchor with `--state` (a real snapshot) / `--pos`, and keep segments short.

## Fuzz

Spec = version-1 Celeste Fuzz JSON (`crates/celeste-fuzz/src/model.rs`; TS types in
`tools/gym-cli/src/fuzzTypes.ts`). Variables are integer ranges (later ranges may reference
earlier variables); inputs are `{keys, at, held_time?, before_input?, after_input?, verify?}`;
`observe_until` frames are simulated; `success` (over `final`) and `checkpoints` filter;
`objectives` (`maximize|minimize|approach`) rank. Expressions are restricted Rhai over
`initial/before/after/final/current` snapshots (`.pos.x .speed.y .state .dashes .stamina
.on_ground .ducking .dead .facing .dash_dir .last_aim .core_mode`), declared variables,
`abs/min/max/sqrt`; compare states as `final.state == state::dash` (no strings).
Direction keys need `held_time` (`"hold::inf"` = until the end); jump defaults to 12 held frames;
grab defaults to held until the end; dash/crouch_dash are 1-frame presses.

```json
{"version":1,
 "variables":[{"name":"dash_at","range":{"from":0,"to":8}},
              {"name":"jump_at","range":{"from":"dash_at + 1","to":"dash_at + 14"}}],
 "inputs":[{"keys":["right"],"at":0,"held_time":"hold::inf","verify":false},
           {"keys":["down","dash"],"at":"dash_at","held_time":1},
           {"keys":["jump"],"at":"jump_at","after_input":["after.speed.x > 200"]}],
 "observe_until":"jump_at + 20",
 "success":["!final.dead"],
 "objectives":[{"type":"maximize","expression":"final.pos.x - initial.pos.x"}],
 "search":{"output":["best","windows","top_3"]}}
```

Result JSON: `best` / `top` (bindings, objective values, verified input frames, final state),
`exactWindows` (successful intervals of the last variable per prefix — the frame windows),
`connectedRegions`, `stats`, and `replay` = the best candidate re-simulated (`summary` + its
inputs as TAS lines). `--candidate '{"dash_at":3,"jump_at":7}'` replays/renders a specific
candidate instead; `--bindings JSON` pins variables. Exit code 2 when no candidate succeeds.

`.ts` specs run directly (tsx): `export default defineFuzz({...})` or a function
`({ api, map }) => spec` (sync/async), plus optional `export const map/room/bin/state`
so the file is self-contained. Examples: `tools/gym-cli/examples/longest-jump.fuzz.ts`,
`tools/gym-cli/examples/playground-hyper.fuzz.json`.

Performance: fuzz drops other rooms' runtime geometry by default (100x+ faster on chapter
maps); add `--with-transitions` only when candidates must enter a neighbouring room.
Candidate counts multiply — keep ranges tight, check `estimatedCandidates`.

## Rendering

Add `--render OUT` (repeatable) to `sim`, `tas`, `fuzz` (renders the replayed candidate) or
`render`. Format by extension:
- `.png` — last frame (`--frame F` picks one), or a **contact sheet** with `--sheet N`
  (N evenly spaced frames) / `--sheet-every K` and `--columns C`. Sheets are the best way for
  an agent to *look* at a run: render one, then open it with the image-reading tool.
- `.gif` (default every 2nd frame at 30 fps) — encoded by **ffmpeg's palettegen/paletteuse**
  when ffmpeg is available (~1.5x faster and ~20x smaller than the bundled encoder: 66 KB vs
  1.6 MB for a 52-frame run), else by the bundled gifenc encoder in one pass with a shared
  palette (no external tools needed).
- `.mp4/.webm/.mkv/.mov` (ffmpeg on PATH, `FFMPEG`, or `--ffmpeg`; 60 fps), or a directory →
  PNG sequence (encoded by the CLI itself: same pixels as Skia, ~10% smaller, with the deflate
  pipelined across frames).
- `--scale N` (default 2 → 640x360), `--camera follow|room` (in-game camera vs whole room),
  `--theme ID` (default: theme extracted for the same map file, else Forsaken City; list with
  `cg themes`), `--hud` (frame/state/input/pos/speed/dash/stamina overlay), `--hitboxes`
  (solids red, entities cyan, hazards magenta, unknown grey, player hitbox green), `--path`
  (player trajectory), `--every/--from/--to/--fps`.
- Room transitions are followed (the renderer switches rooms by `current_room_bounds`).

## TypeScript bindings

`tools/gym-cli/src/index.ts` exports the API (`cg run` passes it as the first argument):

```ts
import type * as Gym from "../tools/gym-cli/src/index.ts";
export default async function (gym: typeof Gym, args: string[]) {
  const map = await gym.openMap("vanilla:1", { room: "1" });   // MapSession
  const trace = await map.simulate("20,R;1,R,J;11,R,J;20,R");  // or SimInput[] / FrameButtons[]
  const run = await map.fuzz(spec, { state: trace.final });     // FuzzRunResult (+ best_inputs)
  await gym.renderTrace(trace, ".tmp/run.gif", { hud: true });
  return { summary: await trace.summary(), table: trace.table({ every: 5 }) };
}
```

Also: `map.start({pos, spawn, patch})`, `map.replayCandidate(spec, bindings)`, `map.check(state,
exprs)`, `map.audit()`, `map.viewFor(state)`, `trace.states/inputs/final/summary()/table()/toJSON()`,
`checkMap`, `resolveTasFile`, `parseTasLines`, `tasFramesToInputs`, `inputsToTas`,
`SceneRenderer`, `contactSheet`, raw `simulateRaw/fuzzRaw/decodeRoom/auditMap/listRooms`.

### `cg run` scripts: name them `.mts`, or they get transpiled as CommonJS

`cg run <script>` imports the file dynamically, and the script's module format comes from its
nearest `package.json` — this repo has **no root `package.json`**, so a plain `.ts` script saved
at the repo root or in `.tmp/` (or in any package without `"type":"module"`) is transpiled as
**CommonJS** instead of ESM. Two things then go wrong:

- Interop breaks: `import gifenc from "gifenc"` binds the default to a bare function, so
  `GIFEncoder` is undefined → **`GIFEncoder is not a function`** (verified: the same import works
  in a `.mts` script). Use `await import("gifenc")` if you must stay in `.ts`.
- Dependencies load through the CommonJS require graph, so module-level state can land in a
  *second copy* of a module: `setRenderBackend()` then configures a different renderer instance
  than the one `gym.renderTrace` / the CLI use, and the script renders through the browser backend
  (`document is not defined`) or draws nothing.

Fix: name the script `script.mts` (or `.mjs`, or keep it inside `tools/gym-cli/`, which is
`"type":"module"`). A default export is still the entry point:
`export default async (gym, args) => {...}`. Reach for `.mts` whenever the script imports anything
from the repo — a sweep script written as `.ts` whose `setRenderBackend` silently does nothing is
a known time sink.

## Fidelity caveats (state them when reporting results)

- Physics is `source_informed_subset` (see `docs/architecture.md`, `docs/tas-replay.md`):
  vanilla traces are mostly frame-exact but not 1:1; unsupported entities are ignored
  (`cg check` lists them); intro/cutscene states are not simulated.
- The simulator does not know about menus, chapter transitions, or Everest helpers beyond the
  decoded entity set. Claims about the real game need the real E2E harness
  (`scripts/e2e-real-collector.mjs`), not this CLI.

## Troubleshooting

- `WASM bundle not found/outdated` → `node scripts/build-wasm.mjs` (needs
  `wasm32-unknown-unknown` + matching `wasm-bindgen-cli`).
- `zip contains N maps` → pass `--bin <substring>` (see `cg maps Mod.zip`).
- `room "x" not found` → names come from `cg maps <map>`; `lvl_` prefixes are stripped.
- `player state X is parsed but not implemented` → the start/trace entered an unsupported
  state; start elsewhere or shorten the segment.
- `ffmpeg not found` → install ffmpeg / pass `--ffmpeg`; `.png` and directories need none, and
  `.gif` falls back to the bundled encoder automatically.
- Rendered PNG/GIF is background + HUD only (no tiles, no player) → a backend returned a
  not-yet-decoded image from `freeze`; surfaces that need a decode belong in `freezeAsync`
  (`web/src/render/canvasBackend.ts`), and the callers must await `prepareGameAssets` /
  `SceneRenderer.prepare`. `npm test` in `tools/gym-cli` covers both.
- Renders are slow, or OOM on a small machine → something is drawn straight out of an offscreen
  canvas. On `@napi-rs/canvas` that copies the whole source surface on *every* `drawImage`
  (~18 MB per 8x8 tile out of the 1024x4600 gameplay atlas). Keep the atlas a decoded image and
  bake composed surfaces once (see the two entries above).
