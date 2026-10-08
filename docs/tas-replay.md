# TAS replay fidelity environment (vanilla 202 / 100%)

Objective: run the **vanilla Celeste full-game TAS** as a fidelity gate for `celeste-physics`
("next-gym"). The real game produces the ground truth; next-gym must reproduce it frame by frame.

## Status and handoff

**Where it stands.** The environment works end to end and the gate is real: the real game plays the
pinned vanilla 202-berry TAS, the instrumented CelesteTAS dumps one record per executed frame, and
`tas_fidelity` replays every room segment through `Simulator` and stops at its first divergence.

| trace | `ok` rooms | mismatch | unsupported | replayed frames | frame-exact |
| --- | ---: | ---: | ---: | ---: | ---: |
| `trace-202-v4` | **395** | 1,072 | 0 | **127,369** | **126,267** |
| `trace-100pct-v4` | **234** | 684 | 0 | **73,085** | **72,386** |
| `trace-1a-v4` | **16** | 4 | 0 | 2,126 | 2,122 |

Starting point was 45 `ok` rooms / 36,253 replayed frames / 87 `unsupported`. The three pending workstreams on the previous revision of this section (Resort clutter with per-`sid` `oshiro_clutter_cleared_*` threading, `CrushBlock`/`DashBlock` with `OnDashCollide`, and the `LevelData` 184 -> 180 clamp) are now **landed** as `f735c98`, `b862df8` and `a27c8b4`; combined they took the 202 trace from 355 to 395 `ok` rooms and 121,264 to 127,369 replayed frames, with 128 segments improved and exactly 3 documented trade-offs: `5-MirrorTemple|0|b-14` (a `permanent` `DashBlock` in `Session.DoNotLoad`, which a Player-only trace cannot express) and `9-Core|1|c-08` x2 (the clamp moves `Bounds.Bottom` 4 px into `Player.CameraTarget`, and the residual is the camera model, not the clamp). Every landed fix cites a
`Player.cs`/`Monocle` line and was proved to be zero-regression with a per-segment diff keyed by
`(sid, mode, room, startRow)`.

**In flight when this section was written** (each in its own git worktree under
`D:\celeste-research\.tmp\wt\`, none of it in master yet — recovering it is the first job):

* `mapdecode` — `CrushBlock`/`DashBlock` decoded as Solids plus the full `OnDashCollide`/`Rebound`
  path, and the `LevelData` 184 → 180 room-height clamp. Measured on `trace-202-v3`:
  `ok 332 → 346`, frames `112,406 → 114,382`, with **3 regressions**, which is why the author
  refused to commit it and stashed the work (`stash@{0}` on base `47b4ba6`). The split I asked for:
  one commit for the block decode (prove zero regressions), a second for the clamp, whose measured
  cost is `9-Core|1|c-08` ×2 (the clamp moves `Bounds.Bottom` 4 px → `Player.CameraTarget`,
  `Player.cs:857`, → a camera-gated spinner flips). `5-MirrorTemple|0|b-14` is a `permanent`
  `DashBlock` already in `Session.DoNotLoad` and **cannot be represented in a Player-only trace**;
  record it as a known limitation, not a regression.
* `posspeed` — `ClutterBlockBase` as a Solid (Celestial Resort clutter, `ClutterBlockGenerator.cs:136-138`;
  `3-CelestialResort` alone holds 34 segments of its cluster) plus a ~5-line harness change threading
  `oshiro_clutter_cleared_0/1/2` across segments by `sid`. Without the threading it measured
  `+2,987 frames` with 14 regressions, all in rooms whose clutter was already cleared by an earlier
  `ClutterSwitch` down-dash (`ClutterSwitch.cs:131-156`). Commit is `a8ef7a6`, based on stale master
  `47b4ba6`; it must be rebased onto `ff50e21` or it will revert the exporter docs and `field-map.md`.

**Structural gaps — these are not "a few more formulas":**

1. **The gate re-anchors per room segment; nothing runs the TAS continuously.** Room transitions,
   cross-room `Session`, chapter chaining and the 22,819 non-`Level` rows have never been executed.
   This is the single biggest gap against "the 202 TAS runs through".
2. **No `Session` model at all** — no berries, checkpoints, area identity, `Session.Flags`
   (`oshiro_clutter_cleared_*`), or `Level.Frozen`.
3. **86 of 162 `PlayerSnapshot` fields are never restored** — the checked-in
   `tools/tas-fidelity/field-map.md` is that list, regenerable with `--dump-field-map`.
4. **166 vanilla entity names still decode to `EntityKind::Unknown`** — no solid, no diagnostic.
   `map.rs` also does not decode the room `space` attribute (only `9-Core`/`9H-Core` are `true`).
5. **The real game is not bit-reproducible.** Two runs of the *same* exporter differ in ~28k rows,
   always starting at `7-Summit|a-00-intro`'s `StDummy` dummy walk (whole-pixel offsets with
   bit-identical `Speed`/`movementCounter`). "Frame-for-frame identical to vanilla" therefore has a
   noise floor that this environment cannot go below.

**How to continue.** Read the sections below for the environment, the ground rules for reading the
artifacts (never whole-file `JSON.parse` a trace), and the reproduce commands. New mechanics should
follow the established loop: pick a cluster from the gate report, cite the `Player.cs` line, prove
zero per-segment regressions, commit in a worktree, integrate.

**The dominant remaining mechanism is a 1-pixel rounding difference.** `tools/tas-fidelity/lib/worklist.mjs` groups every `mismatch` segment of a report into classes (run it as `node tools/tas-fidelity/lib/worklist.mjs <report.json> <out.md>`; it streams). On the `395 ok` master the two biggest classes are `pos|anchor=StNormal` (180 segments / 11,340 frames) and `pos|anchor=StDash` (131 / 8,777), and their position deltas are overwhelmingly **one pixel on one axis** - 142 of the 180 are `(0,+-1)` or `(+-1,0)`, and 82 of the 131 likewise. That is the signature of sub-pixel remainder drift that stays invisible while the gate ignores `movementCounter` and only surfaces when it flips a `Math.Round` step, so these two classes almost certainly share a single root cause in the pixel-move / collision boundary code rather than hundreds of independent bugs. `dashes|anchor=StNormal` (43 / 3,708) is different: every one of its deltas is `(0,0)`, i.e. the divergence is reachable only through the dash count, not through motion.

## Which TAS is "the 202 TAS" — verified, not assumed

Everest CI (`EverestAPI/Everest/.github/workflows/tas-sync-check.yml`) pins
`VampireFlower/CelesteTAS`. Both full-game drivers were run here and their in-game save files were
parsed (not CelesteTAS self-reporting):

| file | declared `FileTime` | save `Time` (ticks) | save berries | save goldens | trace rows (frames fed) | resolved frames | wall clock |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `0 - 100%.tas` | `1:08:45.679(242687)` | `41256790000` = **1:08:45.679 exact** | **176** | 0 | 281,113 | 281,105 | 100.5 s |
| `0 - 202 Berries.tas` | `1:53:44.259(401427)` | `68242590000` = **1:53:44.259 exact** | **202** | 26 | 461,122 | 461,114 | 195.7 s |

"trace rows" is what a real Celeste + CelesteTAS run actually fed to the game; "resolved frames" is
what `tools/tas-replay` computes from the `.tas` tree alone. They differ by exactly **8** on both
files — the `SelectCampaign` name-entry cursor-movement frames in `StartFullGameFile.tas`, whose
geometry needs `ActiveFont.Measure` and the per-language `name_letters` dialog and therefore is not
expressible in a `.tas` file. Everything else (all 8 `SaveAndQuitReenter` expansions, `Repeat`/`Read`
ranges, `StunPause`, `AutoInput`, every input entry) matches frame for frame. For `1A.tas` the two
agree **exactly** at 3,215.

The trace's own frame index `f` has one legitimate gap per file (`SaveAndQuitReenter` jumps
`CurrentFrameInTas` forward inside a single `AdvanceFrame`, `InputController.cs:178-184`), which is
why `max(f) - rows == 11`.

The `FileTime: h:mm:ss.mmm(N)` header is **not** the TAS frame count: `N` is derived from
`SaveData.Time` — the in-game chapter timer, which only advances on `Level` scenes with no pause
overlay open — so it under-counts. It is 242,687 vs 281,113 fed frames for `0 - 100%.tas`.

Both reach all 27 chapter/modes, 24 crystal hearts, 8 cassettes and complete Farewell, but
**only `0 - 202 Berries.tas` reaches the 202/202 berry cap**. So:

* **primary fidelity target = `0 - 202 Berries.tas`** (this is literally "原版 202"),
* `0 - 100%.tas` is kept as the Everest-CI-pinned secondary target.

Source tree (outside the repo): `D:\celeste-research\.tmp\tas\CelesteTAS\CelesteTAS-<rev>\`
(rev `074e71a93a073ec8940d45161c76484a33684841`; Everest `dev` currently pins
`e34fe8e878508f255d6718f3cec9e6145313f7dd`, same layout).

Both files are `Read`-tree drivers — the driver itself has only 61 / ~90 literal input lines; the
rest is pulled in from per-chapter and `Load*` helper files. `tools/tas-replay` flattens that tree
into a canonical per-frame input stream; the real-game trace records the same frames independently,
which is how the resolver is validated.

## Environment (reproducible)

| item | value |
| --- | --- |
| game install | `D:\celeste-research\.tmp\tasrun\game\` — copy of `vendor/celeste-game`, `Mods\` wiped down to CelesteTAS only |
| trace install | `D:\celeste-research\.tmp\tasrun\game-trace\` — same, plus the patched CelesteTAS below |
| Celeste / Everest | Celeste `1.4.0.0-fna`, Everest `1.6418.0` (`6418-azure-19d6c-stable`), .NET `8.0.19` |
| CelesteTAS | built from `MicroblocksQolUtils\.work\celestetas-src` @ `fd1e267`, `dotnet build -c Release` (~16 s), installed as a **dev mod** (`Mods\CelesteTAS-EverestInterop\`) |
| driver | `--sync-check-file "<tas>" --sync-check-result "<result.json>" --loglevel info`, `EVEREST_SAVEPATH` pointed at a freshly recreated `sync-check-saves\` |
| runner | `D:\celeste-research\.tmp\tasrun\run-tas.ps1` (stock mod) and `run-trace.ps1` (trace mod) |

### Known upstream quirk: the game does not self-exit

`SyncChecker.ReportRunFinished` calls `Environment.Exit(0)`, but on this machine that call never
terminates the process (reproduced on every run, including `--headless`): `log.txt` stops dead right
after `Finished check for file`. `result.json` still reaches `finished: true` first.
Any automation must **poll `result.json` for `finished: true`, wait a grace window, re-verify the live
PID's image path, then `Stop-Process` that one PID** — never `WaitForExit` on the SyncChecker itself,
and never kill by process name.

### Run the game on local1 / local2, never in the coordinating session

`local1` (`win_dsh-test1`, Windows account `mb-cloud\dsh-test1`) and `local2` (`win_dsh-test2`) are
**additional accounts on this same machine**, not remote hosts, so they see the very same paths
(`D:\celeste-research\...`). Celeste must be launched from one of them:

```powershell
# from a borrowed local1/local2 account (note: `pwsh` is not on those accounts' PATH, use `& <script>`)
& D:\celeste-research\.tmp\tasrun\run-trace-env.ps1 `
    -TasFile "D:\celeste-research\.tmp\tas\CelesteTAS\<rev>\0 - 202 Berries.tas" `
    -Tag 202 -TraceOut "D:\celeste-research\.tmp\tasrun\trace-202-<tag>.jsonl" `
    -SavesDir "D:\celeste-research\.tmp\tasrun\game-trace\saves-<account>" `
    -ResultOut "D:\celeste-research\.tmp\tasrun\result-<tag>.json"
```

`run-trace-env.ps1` takes the **plain TAS entry** (`1A.tas`, `0 - 202 Berries.tas`), not one of the
pre-made `_trace-*.tas` wrappers, because the trace destination is a `TasFrameTrace,<path>` line
*inside* the `.tas` file and therefore not a process argument. The script generates a per-run wrapper
next to the entry (the resolver resolves `Read` relative to the reading file) and points it at
`-TraceOut`. TAS command arguments must use `/`, never `\` — `CommandLine.TryParse` treats `\` as an
escape. Each account needs its own `-SavesDir` so concurrent runs cannot share save state.

Measured: `trace-1a` takes ~30 s of wall clock on local1 (a cold first run can take ~5 min while the
shader cache warms).


## Architecture

```
                real Everest + CelesteTAS            next-gym (Rust)
                ────────────────────────             ────────────────
0 - 202 Berries.tas ─► CelesteTAS resolves Read/Repeat ─► ground-truth JSONL trace
                        │                                   │
                        │                                   ▼
                        └──────────► canonical input stream ──► segment by (SID, mode, room)
                                                                      │
                                                                      ▼
                                                    celeste-physics replay from the
                                                    trace's first-frame snapshot
                                                                      │
                                                                      ▼
                                                    per-frame diff report
```

The full-game TAS cannot be replayed inside `celeste-physics` as one run: next-gym has no overworld,
no menus, no chapter transitions and only a subset of player states. Instead the TAS is used as a
**corpus of real-game frame sequences**:

1. The real game plays the whole TAS once and emits one JSON record per executed TAS frame.
2. Records are grouped into maximal runs of `(area SID, mode, room)`.
3. For each run, next-gym is initialised from the run's first-frame snapshot (full `Player` field
   restore, the mechanism `examples/compare_real_trace.rs::to_snapshot` already uses) and is fed that
   run's inputs.
4. Every frame of the run is diffed. The headline metric is `exactPrefixFrames` — how many frames
   from the segment start stay identical to the real game.

This directly measures the per-room physics fidelity the objective targets, and divergences point at
concrete mechanics to fix.

### Ground rules for anything that reads these artifacts

The canonical ground-truth traces are **0.9–1.7 GB each and are line-delimited on purpose**:

```
trace-202.jsonl       1480 MB      trace-202-v2.jsonl    1594 MB      trace-202-v3.jsonl    1621 MB
trace-100pct.jsonl     899 MB      trace-100pct-v2.jsonl  968 MB      trace-100pct-v3.jsonl  984 MB
```

Never `JSON.parse(fs.readFileSync(...))` one of them. A scratch `node -e` that did exactly that grew
V8 to **66–74 GB of private memory**, exhausted physical RAM, and the resulting hard-fault storm froze
keyboard and mouse input on the machine for minutes. Reproducing those traces costs real game runs, so
they are never deleted to make room — the readers are fixed instead.

`tools/tas-fidelity/lib/guard.mjs` provides the three safe readers:

```js
import { readJsonSmall, forEachJsonLine, forEachJsonArrayItem } from './tools/tas-fidelity/lib/guard.mjs';

const report = readJsonSmall('.../fidelity.json');                     // throws above a 64 MB cap
await forEachJsonLine('.../trace-202.jsonl', (row) => { /* one frame */ });
await forEachJsonArrayItem('.../fidelity.json', 'segments', (s) => { /* one room segment */ });
```

`forEachJsonArrayItem` is a scanner, not a parser: it tracks brace and string state and hands over one
complete element at a time, so a report whose `segments` array is hundreds of megabytes is still safe
to walk.

## Components

| path | role |
| --- | --- |
| `tools/tas-replay/` | flattens a `.tas` `Read`/`Repeat` tree into the canonical input stream (`src/resolve.mjs`), with `docs/tas-format.md` |
| `tools/tas-fidelity/` | renders the Rust fidelity report into Markdown, and holds `lib/guard.mjs` |
| `crates/celeste-physics/examples/tas_fidelity.rs` | replays every level segment of a trace through `Simulator` and reports divergences |
| `tools/celestetas-trace/` | the `TasFrameTrace` instrumentation for CelesteTAS, plus `apply.mjs` |
| `D:\celeste-research\.tmp\tasrun\run-trace-env.ps1` | launches the game from local1/local2 and owns the PID lifecycle |

## Ground-truth trace format

One JSON object per line, one line per executed TAS frame:

```jsonc
{"n":1,"f":2,"fi":2,"line":2,"dt":0.016666699200868607,"rawDt":0.016666699200868607,"timeRate":1,
 "in":{"mx":0,"my":0,"gly":0,"jump":false,"jumpP":false,"dash":false,"dashP":false,
       "cdash":false,"cdashP":false,"grab":false,"talk":false,"talkP":false,
       "aim":[0,0],"feather":[0,0]},
 "a":0,"aStr":"   1","nf":1,"rep":[0,0],
 "scene":"Level","sid":"Celeste/1-ForsakenCity","area":1,"mode":0,"room":"1","deaths":0,
 "levelTime":0,"state":"StIntroJump",
 "p":{"Speed":[0,0],"Position":[19,144],"Dashes":1,"Stamina":110,"onGround":true, ...}}
```

* `n` monotonic row index, `f` `CurrentFrameInTas`, `fi` `CurrentFrameInInput`, `line` studio line.
* `in` is the **live virtual input state read back from the game** after the frame, i.e. what the game
  actually consumed. Reliable for everything except `dashP`/`cdashP` during the `Boost` state, where
  `Player.BoostUpdate` calls `Input.Dash.ConsumePress()` (`Player.cs:4724`).
* `a` / `aStr` are the TAS input frame's raw `StudioCommunication.Actions` bitmask and source text —
  the authoritative, gameplay-independent record of the intended input.
* `p` contains **every instance field reachable from `Player` by walking the base chain**
  (`Player` → `Actor` → `Platform` → `Entity`), 126 fields, with round-trip float formatting, so a
  snapshot restore can be exact instead of lossy. `Position` (on `Entity`) and `movementCounter`
  (on `Platform`) matter most: without `movementCounter` a segment anchored mid-motion starts with a
  zero sub-pixel remainder and drifts a pixel on its very first frame.
* Non-level scenes (overworld, transitions, vignettes) still produce a row, with `scene` naming them.

## Measured fidelity — the gate

`crates/celeste-physics/examples/tas_fidelity` replays every level segment of a trace through
`Simulator` and stops each segment at its first divergence. `replayed frames` is therefore the
headline progress metric: an improved mechanic keeps more segments alive for longer.

| trace | date | segments | `ok` | `mismatch` | `unsupported` | replayed frames | exact frames |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `trace-1a` | first baseline | 20 | 1 | 18 | 1 | 567 | 548 |
| `trace-1a` | after JumpThru fix | 20 | 1 | 18 | 1 | 741 | 722 |
| `trace-1a` | after the parallel workstreams | 20 | **8** | 12 | **0** | **1,686** | **1,674** |
| `trace-100pct` | first baseline | 918 | 29 | 840 | 48 | 20,609 | 19,721 |
| `trace-100pct` | after JumpThru fix | 918 | 30 | 840 | 48 | 21,653 | 20,765 |
| `trace-202` | first baseline | 1,468 | 44 | 1,335 | 87 | 34,223 | 32,801 |
| `trace-202` | after JumpThru fix | 1,468 | 45 | 1,335 | 87 | 36,253 | 34,831 |
| `trace-202` | after the parallel workstreams | 1,468 | **159** | 1,308 | **0** | **67,527** | **66,219** |
| `trace-1a` | after wall-jump/retention/slide | 20 | **14** | 6 | 0 | **2,084** | **2,078** |
| `trace-202` | after wall-jump/retention/slide | 1,468 | **234** | 1,233 | 0 | **81,380** | **80,147** |
| `trace-202` | after the second wave (v1 trace) | 1,468 | **304** | 1,163 | 0 | **102,004** | **100,841** |
| `trace-202-v3` | same build, richer trace | 1,468 | **332** | 1,135 | 0 | **112,406** | **111,271** |
| `trace-100pct-v3` | same build | 918 | **220** | 698 | 0 | **67,834** | **67,136** |
| `trace-1a-v3` | same build | 20 | **16** | 4 | 0 | **2,126** | **2,122** |
| `trace-202-v3` | after the v3/v4 exporter (pre-consumption `*P0`, `coreMode`, `inSpace`) | 1,468 | **353** | 1,114 | 0 | **119,918** | **118,804** |
| `trace-202-v4` | same build, v4 trace | 1,468 | **355** | 1,112 | 0 | **121,236** | **120,124** |
| `trace-100pct-v4` | same build | 918 | **234** | 684 | 0 | **73,070** | **72,386** |

The exporter wave was verified independently against the `trace-202-v3` baseline: **61 segments
improved, 0 regressed, 0 missing**. `trace-100pct-v4` and `trace-202-v4` were produced from **local1**
(1.04 GB / 1.71 GB, both `sync-check status: success`, row counts identical to v1: 281,113 and
461,122). Independent streaming validation of the v4 traces: `coreMode ∈ {0,1,2}` present,
`inSpace == true` on 1,385 / 2,142 Level rows, and `in.*P0 != in.*P` on **175 / 285 rows** — exactly
the frames where the game consumed the press before the trace read it.

The second wave was verified with a per-segment diff keyed by `(sid, mode, room, startRow)` against
the `trace-202` v1 baseline: **242 segments improved, 0 regressed, 0 missing**. The v3 trace scores
higher because it exports and restores fields the v1 trace never carried.

94–96% of every replayed frame is already frame-exact; the gate's value is that each remaining
divergence names a specific mechanic.

### Fixed so far

Work was split into one workstream per divergence class, each in its own git worktree and verified
against this same gate:

* **`JumpThru` collider height** (`5299096`). `Celeste.JumpthruPlatform` forwards only
  `data.Position` and `data.Width` to `JumpThru`, whose constructor replaces the map entity's 8 px
  height with `new Hitbox(width, 5f)` anchored top-left (`JumpThru.cs:12`,
  `JumpthruPlatform.cs:23-26`). Vanilla `jumpThru` fell through the generic bounds arm and kept the
  raw 8 px, so the collider reached 3 px lower than the real game and the `Player.Update` JumpThru
  Assist (`Player.cs:1787-1790`) fired on frames the real game skips.
* **`NormalBegin`'s `maxFall` reset and the held-down wall-slide guard** (`586011d`). Every
  `DashUpdate` branch that returns state 0 runs `NormalBegin`, which resets `maxFall = 160f`
  (`Player.cs:3531-3534`); the simulator kept the cap the dash inherited. `Player.cs:3749` also gates
  the whole wall-slide fall target on `Input.MoveY.Value != 1`.
* **`Actor.MoveHExact`/`MoveVExact` remainder zeroing** (`95f7cfc`). The source zeroes the *moving
  axis's* `movementCounter` at the collision point, before invoking the callback (`Actor.cs:220`,
  `:249`, `:269`); the simulator zeroed it inside each callback branch, so branches that return early
  kept the pre-collision fraction and the position drifted a pixel later.
* **Dream-dash inventory** (`122491e`). `Inventory.DreamDash` is session state the trace cannot
  export; it is recovered from the surviving witness `Player.dreamDashCanEndTimer`, which is only
  written by `DreamDashBegin` (`Player.cs:5144`) behind `Player.DreamDashCheck` (`Player.cs:3420`).
* **The four `Player.Intro*` states** (`321361e`) — `IntroWalk`, `IntroJump`, `IntroWakeUp`,
  `IntroThinkForABit` (`Player.cs:5969-6174`) and `IntroRespawn`'s tween clock. This removed the last
  `unsupported` segments from the 202 trace (87 → 0). It also added
  `LevelLoader`'s 3-cell outward room-edge tile bleed (`LevelLoader.cs:233-263`).
* **The `Player.cs:1791-1794` dash down-close** and the `DashUpdate` buffered-jump block
  (`Player.cs:4384-4441`), `DashCorrectCheck` (`Player.cs:4191-4209`), and the derivation of the
  simulator's invented `state_timer` from the exported `dashAttackTimer` (`Player.cs:4276-4304`,
  `1577-1580`).
* **Wall jumps, wall-speed retention and the wall slide** (`52b61a9`). `Math.Sign(0f)` is `0` while
  Rust's `f32::signum(0.0)` is `1.0`, so the zeroed `Speed.X` from a wall collision cancelled the
  `wallSpeedRetention` window instead of restoring the retained speed (`Player.cs:1669`, `1673-1676`);
  `NormalEnd` (`Player.cs:3536-3541`) was not modelled at all; `DashUpdate`'s buffered-jump block is a
  `ClimbJump` or plain `WallJump` for any `DashDir` other than the super cases
  (`Player.cs:4393-4441`); the wall-slide block follows the force-move-adjusted `moveX` into `Facing`
  with a live `wallSlideTimer` (`Player.cs:3749-3771`); the climb drain reads `lastClimbMove`
  (`Player.cs:4045`, `4056-4079`); and the wall boost is consumed before the on-ground stamina reset
  (`Player.cs:1560-1576`).

### Known open gaps (measured, not guessed)

* **`Session.CoreMode` is not exported**, so the Core ice factor
  `if (onGround && level.CoreMode == Cold) num2 *= 0.3f` (`Player.cs:3681-3684`) is implemented but
  inert. Forcing `Cold` measured **+294 replayed frames** across one copy of the Core rooms.
* **`Level.InSpace`** (`Level.cs:449`) is a per-room map property `map.rs` does not decode, so
  `Player.cs:3703-3706`, `3718-3722`, `3778-3781` (`*= 0.6f`) are unimplemented.
* **`Level.Wind`** — 41+ segments diverge by exactly `level.Wind * 0.1 * Engine.DeltaTime` in x, from
  `Player.cs:1180`'s `WindMover` component, which runs before the main `MoveH` (`Player.cs:1801`).
  The v3 exporter now carries it; the restore is landed, and the residual wind segments are next.
* **`Engine.FreezeTimer`** and **`Level.Transitioning`** — the ~40-row transition window at each room
  entry is still unreplayable and each segment anchors after it.
* **`Player.Ducking`** — a computed property over `Entity.Collider`; the trace exports `wasDucking`
  (which equals `Ducking` after each `Player.Update`, `Player.cs:1921-1924`) and X4 measured that
  restoring it is the single biggest remaining win (57 segments: a superslide needs the
  `if (Ducking) { Speed.X *= 1.25; Speed.Y *= 0.5; }` branch, `Player.cs:2495-2502`).
* **The gate does not compare `movementCounter`**, so a 0.667 px remainder drift stays invisible until
  it flips a `Math.Round` step — which is the shape of most remaining "1–2 px `pos`, speed's 7th
  significant digit differs" divergences.
* **`CrushBlock` / `DashBlock` are not decoded at all** (42 segments) — 24×24 crushers arrive as
  `EntityKind::Unknown` with no solid, and `sim.rs` has no `OnDashCollide` path at all
  (`CrushBlock.cs:279`, `Player.cs:2784-2800`, `3168-3169`).
* **166 vanilla entity names decode to `EntityKind::Unknown`** with no solid and no diagnostic.
* **`Session`** — no session model at all (no berries, checkpoints, area identity).
* The three residual `wall_dir`-instead-of-`WallJumpCheck` call sites (RedDash/StarFly/NormalUpdate).



## Captured ground truth

| artifact | rows | Level rows | level segments | size |
| --- | ---: | ---: | ---: | ---: |
| `trace-100pct.jsonl` | 281,113 | 266,262 | 918 | 845 MB |
| `trace-202.jsonl` | 461,122 | 438,303 | 1,468 | 1.39 GB |
| `trace-1a.jsonl` (1A only, fast iteration) | 3,215 | 3,213 | — | 9.8 MB |
Player-state coverage across `trace-202.jsonl` (this is the per-mechanic corpus the fixes will be
driven from):

| state | rows | state | rows |
| --- | ---: | --- | ---: |
| `StNormal` | 250,547 | `StIntroRespawn` | 920 |
| `StDash` | 116,203 | `StFlingBird` | 635 |
| `StDummy` | 16,241 | `StCassetteFly` | 562 |
| `StStarFly` | 10,460 | `StTempleFall` | 414 |
| `StLaunch` | 8,311 | `StBoost` | 304 |
| `StDreamDash` | 6,950 | `StSwim` | 250 |
| `StRedDash` | 6,058 | `StIntroThinkForABit` | 97 |
| `StClimb` | 4,454 | `StHitSquash` | 25 |
| `StSummitLaunch` | 4,248 | | |
| `StIntroJump` / `StIntroWakeUp` / `StIntroWalk` | 3,899 / 2,151 / 1,864 | | |
| `StPickup` | 1,830 | | |
| `StAttract` | 1,578 | | |

## Reproduce

```powershell
# 0. build + install the trace-capable CelesteTAS (once) - patch + build in one step
robocopy "D:\celeste-research\.tmp\tasrun\celestetas-src" "D:\celeste-research\.tmp\tasrun\celestetas-trace" /E /XD .git
node tools\celestetas-trace\apply.mjs "D:\celeste-research\.tmp\tasrun\celestetas-trace"
# then install that tree as a dev mod at <game>\Mods\CelesteTAS-EverestInterop\

# 1. ground truth (real game; ~2 min for 100%, ~3.5 min for the 202 TAS)
D:\celeste-research\.tmp\tasrun\run-trace.ps1 `
  -TasFile "D:\celeste-research\.tmp\tas\CelesteTAS\<rev>\_trace-202.tas" -Tag 202

# 2. canonical input stream
node tools\tas-replay\src\cli.mjs "D:\celeste-research\.tmp\tas\CelesteTAS\<rev>" "0 - 202 Berries.tas" --out .tmp\inputs-202.json

# 3. per-room fidelity report
cargo run -q -p celeste-physics --release --example tas_fidelity -- `
  --trace D:\celeste-research\.tmp\tasrun\trace-202.jsonl `
  --maps vendor\celeste-game\Content\Maps --out .tmp\fidelity-202.json
node tools\tas-fidelity\render-report.mjs .tmp\fidelity-202.json .tmp\fidelity-202.md
```

The wrapper TAS files that switch the trace on must live in the TAS tree itself, because `Read`
resolves relative to the reading file:

```
_trace-202.tas:
TasFrameTrace,D:/celeste-research/.tmp/tasrun/trace-202.jsonl
Read,0 - 202 Berries
```

Paths in TAS command arguments use `/`, not `\` — `CommandLine.TryParse` treats `\` as an escape.
