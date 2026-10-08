# TAS replay fidelity environment (vanilla 202 / 100%)

Objective: run the **vanilla Celeste full-game TAS** as a fidelity gate for `celeste-physics`
("next-gym"). The real game produces the ground truth; next-gym must reproduce it frame by frame.

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

## Components

| path | role |
| --- | --- |
| `tools/tas-replay/` | flattens a `.tas` `Read`/`Repeat` tree into the canonical input stream (`src/resolve.mjs`), with `docs/tas-format.md` |
| `tools/tas-fidelity/` | renders the Rust fidelity report into Markdown |
| `crates/celeste-physics/examples/tas_fidelity.rs` | replays every level segment of a trace through `Simulator` and reports divergences |
| `.tmp/celestetas-patch/TasFrameTrace.cs` | CelesteTAS command `TasFrameTrace,<path.jsonl>` — the ground-truth dump |
| `D:\celeste-research\.tmp\tasrun\run-trace.ps1` | spawns the trace-capable install and owns the PID lifecycle |

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
* `p` contains **every declared instance field of `Player`** with round-trip float formatting, so a
  snapshot restore can be exact instead of lossy.
* Non-level scenes (overworld, transitions, vignettes) still produce a row, with `scene` naming them.

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
# 0. build + install the trace-capable CelesteTAS (once)
robocopy "D:\celeste-research\.tmp\tasrun\celestetas-src" "D:\celeste-research\.tmp\tasrun\celestetas-trace" /E /XD .git
copy "D:\celeste-research\.tmp\celestetas-patch\TasFrameTrace.cs" `
     "D:\celeste-research\.tmp\tasrun\celestetas-trace\CelesteTAS-EverestInterop\Source\Tools\TasFrameTrace.cs"
# plus one line in Source\TAS\Input\InputController.cs next to ExportGameInfo.ExportInfo():
#     TasFrameTrace.ExportInfo();
dotnet build "D:\celeste-research\.tmp\tasrun\celestetas-trace\CelesteTAS-EverestInterop\CelesteTAS-EverestInterop.csproj" -c Release -p:UseSymlinks=false

# 1. ground truth (real game; ~3.5 min for the 202 TAS)
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
