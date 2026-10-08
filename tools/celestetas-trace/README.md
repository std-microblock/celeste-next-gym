# `TasFrameTrace` — per-frame ground-truth export for CelesteTAS

This is the instrumented build of [CelesteTAS-EverestInterop](https://github.com/EverestAPI/CelesteTAS-EverestInterop)
that turns a real-game TAS run into the **frame-by-frame ground truth** the TAS fidelity gate diffs against.

CelesteTAS already has `ExportGameInfo`, but it emits a TSV with a handful of derived strings and no
`Dashes` / `onGround` / `Facing` / full `Player` field set. `TasFrameTrace` emits one JSON object per
executed TAS frame containing **every declared instance field of `Celeste.Player`** (round-trip float
formatting) plus the live virtual input state and the raw TAS input actions.

## What it adds

| file | change |
| --- | --- |
| `CelesteTAS-EverestInterop/Source/Tools/TasFrameTrace.cs` | new file — the exporter and its `TasFrameTrace,<path.jsonl>` / `EndTasFrameTrace` TAS commands |
| `CelesteTAS-EverestInterop/Source/TAS/Input/InputController.cs` | one line: `TasFrameTrace.ExportInfo();` next to the existing `ExportGameInfo.ExportInfo();` |

`ExportInfo()` is called exactly once per TAS frame from `InputController.AdvanceFrame`, immediately
before `InputHelper.FeedInputs` — the same call site as the existing `ExportGameInfo.ExportInfo()`;
the actual write happens in the existing post-`Engine.Update` hook, which is the same point
`ExportGameInfo` records at. `ExportInfo` captures `CurrentFrameInTas` while it still names the frame
about to run and reports `f = CurrentFrameInTas + 1`, because `AdvanceFrame` always ends with exactly
one `CurrentFrameInTas++`.

`f` therefore has **legitimate gaps**: `SaveAndQuitReenter` and `SelectCampaign` jump
`CurrentFrameInTas` forward inside a single `AdvanceFrame`
(`InputController.cs:178-184`), so those frames are never fed to the game and never appear.
In `0 - 100%.tas` there is exactly one such gap — `f` jumps 5457 → 5469 in room `s1` — which fully
explains why `max(f) - rows == 11`.

## Usage

```powershell
# 1. work on a copy; the patch edits the tree it is given
robocopy <celestetas-src> <copy> /E /XD .git

# 2. apply the patch, then build (pass --no-build to skip the build)
node tools\celestetas-trace\apply.mjs <copy>

# 3. install <copy> as a dev mod: <game>\Mods\CelesteTAS-EverestInterop\ (everest.yaml at the root,
#    assembly under bin\). `.everestignore` already hides .git/.github/Studio from Everest.

# 4. drive the TAS with a wrapper that turns the trace on first
#    _trace-202.tas:
#        TasFrameTrace,D:/path/to/trace-202.jsonl
#        Read,0 - 202 Berries
#    Celeste.exe --sync-check-file "<wrapper>" --sync-check-result "<result.json>" --loglevel info
```

Paths in TAS command arguments must use `/`, not `\` — `CommandLine.TryParse` treats `\` as an escape.

## Record schema

```jsonc
{"n":1,"f":2,"fi":2,"line":2,
 "dt":0.0166667,"rawDt":0.0166667,"timeRate":1,
 "in":{"mx":0,"my":0,"gly":0,"jump":false,"jumpP":false,"dash":false,"dashP":false,
       "cdash":false,"cdashP":false,"grab":false,"talk":false,"talkP":false,
       "aim":[0,0],"feather":[0,0]},
 "a":0,"aStr":"   1","nf":1,"rep":[0,0],
 "scene":"Level","sid":"Celeste/1-ForsakenCity","area":1,"mode":0,"room":"1",
 "deaths":0,"levelTime":0,"state":"StNormal",
 "p":{"Position":[19,144],"Speed":[0,0],"Dashes":1,"Stamina":110,"onGround":true, ...}}
```

* `n` row index · `f` `CurrentFrameInTas` · `fi` `CurrentFrameInInput` · `line` studio line
* `in` live `Celeste.Input` state after the frame — what the game actually consumed
* `a` / `aStr` / `nf` / `rep` raw `StudioCommunication.Actions` bitmask, source text, frame count and
  repeat index of the TAS input frame (authoritative and gameplay-independent)
* `p` all instance fields reachable from `Player` by **walking the whole base chain**
  (`Player` → `Actor` → `Platform` → `Entity`, stopping before `object`) with most-derived-wins
  de-duplication, by exact C# name (`Vector2` → `[x,y]`, enums → int). This yields 126 fields and is
  deliberate: `Position` is declared on `Entity` and `movementCounter` on `Platform`, and without the
  latter a segment restored mid-motion starts with the wrong sub-pixel remainder and drifts by a
  pixel on the very first frame.
* non-`Level` scenes still produce a row with `scene` naming them (`Overworld OuiChapterPanel`,
  `LevelEnter`, `AreaComplete`, …), so the trace can be aligned frame by frame

## Caveats

* `in.dashP` / `in.cdashP` read the live `VirtualButton.Pressed`, which
  `Player.BoostUpdate` clears via `Input.Dash.ConsumePress()` (`Player.cs:4724`). Use `a` when the
  exact press edge matters during the `Boost` state.
* The writer buffers and flushes every 512 rows; the buffer is flushed on `DisableRun`. Killing the
  process mid-run can lose the tail — the harness therefore waits for `result.json` `finished: true`.
* The exporter deliberately does **not** record room entity state (zip movers, spinners, bumpers, …).
  Segment replay therefore relies on `Simulator::new` re-initialising entities from the map, which is
  exact for segments that start at a room entry and approximate elsewhere.
