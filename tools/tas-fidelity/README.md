# `tas-fidelity` — replaying a real-game Celeste TAS against `celeste-physics`

This tool answers one question with measurements, not opinions:

> Given the ground truth of a real vanilla Celeste + CelesteTAS run (one JSON object per executed TAS
> frame), where exactly does `celeste-physics` stop reproducing the game frame for frame?

It groups the trace into **room segments** (maximal runs of consecutive rows with the same
`(sid, mode, room)`), restores a `PlayerSnapshot` from the segment's first *live* row, replays the
segment's inputs through `Simulator`, and diffs every produced snapshot against the matching trace
row. The result is a machine-readable JSON report plus a Markdown rendering.

```
trace.jsonl ──► tas_fidelity ──► report.json ──► render-report.mjs ──► report.md
                 (Rust example)     (per segment: exactPrefixFrames + first mismatch)
```

| path | role |
| --- | --- |
| `crates/celeste-physics/examples/tas_fidelity.rs` | the harness (trace → report) |
| `tools/tas-fidelity/render-report.mjs` | `report.json` → Markdown |
| `tools/tas-fidelity/README.md` | this file |

Everything below was produced by the commands in [Reproduce](#reproduce); numbers are copy-pasted
from real output, never estimated.

## CLI

```bash
cargo run -q -p celeste-physics --release --example tas_fidelity -- \
  --trace <trace.jsonl> \
  --maps <repo>/vendor/celeste-game/Content/Maps \
  --out <report.json> \
  [--min-frames 1] [--limit-segments N] [--max-frames N] [--rooms a,b] \
  [--probe-remainder N] [--probe-frames N] [--dump-field-map]
```

| flag | meaning |
| --- | --- |
| `--trace` | ground-truth JSONL (streamed line by line, never loaded whole) |
| `--maps` | directory of vanilla `*.bin` area maps |
| `--out` | report path (parent directories are created) |
| `--min-frames N` | ignore segments with fewer than `N` trace rows (a segment needs at least 2 rows to have one input frame) |
| `--limit-segments N` | stop the scan after `N` simulated segments (early exit; `totals.truncated` is set) |
| `--max-frames N` | cap the rows retained per segment |
| `--rooms a,b` | only simulate segments whose room name is listed |
| `--probe-remainder N` | diagnostic (see [Remainder probe](#remainder-probe)) for the first `N` mismatch segments |
| `--probe-frames N` | frame cap for each probe replay (default 200) |
| `--dump-field-map` | print the generated `PlayerSnapshot` coverage table as Markdown and exit |

## How a segment is replayed

1. **Grouping.** Rows with `scene != "Level"` (overworld, `LevelLoader`, `LevelExit`, …) are skipped
   but counted. Level rows are grouped into maximal runs of identical `(sid, mode, room)`. Rows are
   aligned by row order (`n`), never by `f`: `CurrentFrameInTas` legitimately jumps forward when the
   TAS driver uses `SaveAndQuitReenter`/`SelectCampaign` inside one `AdvanceFrame`
   (`CelesteTAS-EverestInterop/Source/TAS/Input/InputController.cs`).
2. **Area map.** The SID suffix after `Celeste/` names the file, with the side letter inserted
   before the `-` for B/C sides (`1-ForsakenCity` + mode 1 → `1H-ForsakenCity.bin`; `LostLevels` has
   a single file and is always mode 0). `celeste_map_rooms` validates that the room exists, then
   `decode_map_room(bytes, Some(room))` selects it. Failures become `map_error`, never a panic.
   Decoded maps are cached per `(file, room)`.
3. **Anchor row.** `Player.Update` decrements `StrawberryCollectResetTimer` unconditionally, so two
   consecutive rows with an identical value prove `Player.Update` did **not** run on that engine
   frame. The first row of the segment for which that is false — and which actually carries a
   `state`/`p` snapshot — is the *anchor*; its full `p` object is restored into a `PlayerSnapshot`
   and `Simulator::new` is called. Inputs come from the rows after the anchor, so replaying row `k`'s
   input reproduces row `k`.
4. **Per-frame diff.** `tol = 0.01` on `pos`, `speed`, `stamina`; exact equality on `state`,
   `facing`, `dashes`, `on_ground`, `dead`. `exactPrefixFrames` counts consecutive matching frames
   from the anchor; the first frame with a non-empty reason list is captured (both snapshots) and
   the segment stops. `SimulationError::UnsupportedState` is recorded with the state name and frame
   offset; unknown state names become `state_map_error`. Nothing panics.

### Rows where `Player.Update` did not run

Two engine mechanisms make a trace row a stale copy of the previous one:

* **Room transition.** `Level.Update` takes the `Transitioning` branch and updates only
  `Tags.TransitionUpdate` entities plus the transition coroutine, which drives
  `Player.TransitionTo` (`Player.cs`) directly — `Player.Update` never runs. This is the
  `leadingSkippedFrames` run at the head of almost every segment (exactly **40** rows for most
  Forsaken City transitions in `trace-1a.jsonl`, e.g. rows 181–220 before room `2`; the count scales
  with the room's transition duration). Those rows are **not replayed**: the trace does not export
  the transition coroutine state (`Session.Transitioning` / `Level.transition` are not fields on the
  player's base chain), so the simulator cannot be told that a transition is in progress. The next
  live row is a complete post-transition snapshot and becomes the anchor.
* **`Celeste.Freeze(t)`.** `Monocle.Engine.Update` skips `Scene.Update` entirely while
  `Engine.FreezeTimer > 0`, so the player does not move. The simulator models this itself
  (`Simulator::step` returns early while `snapshot.freeze_timer > 0`, exactly like the engine) and
  models the `Celeste.Freeze` call sites too (`Player.DashBegin` → `0.05`, `Refill` → `0.05`,
  `HeartGem` → `0.2`, …). Mid-window stalled rows are therefore **still stepped**: that keeps the
  `MInput`-equivalent `VirtualButton` buffers decaying during the freeze, which matters — an earlier
  revision that skipped those rows let a buffered dash fire four frames late and produced a bogus
  state mismatch. `freezeDisagreementFrames` counts frames where the simulator's freeze state
  disagreed with the trace: **0 for every segment of `trace-1a.jsonl`**, and 181 of 21,653 replayed
  frames (0.8%) over the whole 100% run. `freeze_timer` itself (`Engine.FreezeTimer`) is not
  exported, so it is never restored — it is only *reproduced* by the simulator's own freeze model and
  checked against the trace's stalled rows.

### What is compared, and what cannot be

| quantity | source | compared |
| --- | --- | --- |
| `pos` | `p.Position` (`Monocle.Entity.Position`) | `tol 0.01` |
| `speed` | `p.Speed` | `tol 0.01` |
| `stamina` | `p.Stamina` | `tol 0.01` |
| `state` | top-level `state` (`PlayerStates.GetCurrentStateName`, 26 names from the `Player.St*` ids in `Player.cs`) | exact, exhaustive mapping |
| `facing` | `p.Facing` (`Facings`: `-1` Left, `1` Right) | exact |
| `dashes` | `p.Dashes` | exact |
| `on_ground` | `p.onGround` **vs `PlayerSnapshot::player_on_ground`** | exact |
| `dead` | `p.<Dead>k__BackingField` | exact |
| `ducking` | top-level `ducking` (`Player.Ducking`, v2 traces only) | **diagnostic only** (`duckingDisagreementFrames`) |
| `wind` | top-level `wind` (`Level.Wind`, v2 traces only) | **diagnostic only** (`windDisagreementFrames`) |
| `state_timer` | — | **not comparable** |
| `camera` | — | **not comparable** |
| `freeze_timer` | top-level `freezeTimer` (`Engine.FreezeTimer`, v2 traces only) | restored at the anchor; mid-segment agreement via `freezeDisagreementFrames` |

Three details that are easy to get wrong:

* **`on_ground` maps to `player_on_ground`, not to `PlayerSnapshot::on_ground`.** `p.onGround` is
  `Celeste.Player.onGround`, the source-private field that `Player.Update` writes behind a
  `Speed.Y >= 0f` gate (`Player.cs` L1504-1526). `PlayerSnapshot::player_on_ground` is its mirror
  (`sim.rs` gates it identically), while `PlayerSnapshot::on_ground` is the simulator's own
  *post-entity geometric* probe, which is a different quantity by design. Comparing the trace's
  `onGround` against the geometric value manufactures false mismatches on a rising player resting
  near a ledge; `geometricGroundDiffFrames` reports how often the simulator's two ground values
  disagree, as a diagnostic only (1,637 over the 202 run before the exporter tail, 2,791 after —
  the increase is the extra frames the newly restored fields keep alive, not a new divergence).
  The distinction is load-bearing for wind too: `Player.WindMove` gates its horizontal push on
  `Ducking && onGround` (`Player.cs:3095`), i.e. on the *field*, which is
  `PlayerSnapshot::player_on_ground` at the moment `WindController.Update` runs.
* **`rawDt`, not `dt`, is fed to `InputState::frame_delta_time_bits`.** Monocle computes
  `Engine.DeltaTime = RawDeltaTime * TimeRate * TimeRateB`, and `Simulator::step` multiplies the
  supplied bits by `snapshot.time_rate` — so the *raw* delta is the correct input. In
  `trace-1a.jsonl` `timeRate` is `1` on every row and `dt == rawDt` bit for bit, so the two choices
  are indistinguishable there; they are not in general (HeartGem writes `Engine.TimeRate`).
* **`ducking` is restored, but its evolution is only 99.2% right.** `Player.Ducking` is
  `Collider == duckHitbox || Collider == duckHurtbox` (`Player.cs` L1005-1028), and `Collider` lives
  on `Monocle.Entity`, so it is not a declared field on the player's base chain. The v2 exporter
  writes `ducking` plus the active collider rect (`collider`) as top-level keys, so the anchored
  hitbox is now exact; `duckingDisagreementFrames` counts the replayed frames where the simulator's
  own setter rules still disagree (319 of 40,989 replayed frames on `trace-202-v2`).

### The exporter tail (v2 traces)

Six divergences turned out to be unreachable from the player's field chain because the ground truth
did not export the quantity at all. `tools/celestetas-trace/TasFrameTrace.cs` now appends a
**tail** to every `scene == "Level"` row — after every pre-existing key, so nothing that reads the
older traces changes meaning:

| key | C# source | restored into | why it cannot be inferred |
| --- | --- | --- | --- |
| `wind` | `Celeste.Level.Wind` (`Level.cs:149`) | `PlayerSnapshot::wind` (verbatim, at the anchor) | `WindController.Update` rewrites it every frame with `Calc.Approach(level.Wind, targetSpeed, 1000f * Engine.DeltaTime)` (`WindController.cs:194`) and then displaces the Player's `WindMover` component (`Player.cs:1180`) by `level.Wind * 0.1f * Engine.DeltaTime` (`WindController.cs:199-201`). The ramp crosses room boundaries, so a segment anchored mid-flight cannot reconstruct it. |
| `windTarget` | `WindController.targetSpeed` (`WindController.cs:47`, reflection through the private `Level.windController` field, `Level.cs:101`) | `PlayerSnapshot::wind_target` | the ramp needs the source's own target |
| `windPattern` | `WindController.pattern` (`WindController.cs:45`) | — (evidence) | |
| `windSine`, `windSineTimer` | `Level.cs:151-153` | — (evidence) | visual only |
| `transitioning` | `Celeste.Level.Transitioning` (`Level.cs:221`) | — (evidence) | `PlayerSnapshot::transition_timer` is a countdown, not the `transition != null` predicate |
| `freezeTimer` | `Monocle.Engine.FreezeTimer` (`Monocle/Engine.cs:28`) | `PlayerSnapshot::freeze_timer` | while positive `Engine.Update` only decrements it and skips `Scene.Update` (`Engine.cs:266-269`) |
| `ducking` | `Player.Ducking` (`Player.cs:1005-1028`) | `PlayerSnapshot::ducking` | computed property over `Monocle.Entity.Collider` (`Monocle/Entity.cs:73`) |
| `collider` | `Collider.AbsoluteLeft/AbsoluteTop/Width/Height` (`Monocle/Collider.cs:229,205,12,14`) | — (evidence; `[x,y,w,h]`) | |
| `inventory` | `Celeste.Session.Inventory` (`Session.cs:35`, `PlayerInventory.cs:22-28`) | `PlayerSnapshot::can_dream_dash` <- `inventory.DreamDash` | session state; `Player.Inventory` forwards it (`Player.cs:956-966`) and the source reads it at `Player.cs:3420,4500`. The old harness could only infer the flag from `Player.dreamDashCanEndTimer`, which is wrong for a session that has not dream-dashed since the last respawn. |

`Session.Level` and `Session.Deaths` already had their own keys (`room`, `deaths`) and were not
duplicated. `ducking`/`collider` are absent only on rows where `level.Tracker.GetEntity<Player>()`
is null — the same rows that have no `state`/`p` (234 of 266,262 Level rows on the 100% trace, 302
of 438,303 on the 202 trace).

The three traces were regenerated as **`trace-1a-v2.jsonl`** (3,215 rows),
**`trace-100pct-v2.jsonl`** (281,113 rows) and **`trace-202-v2.jsonl`** (461,122 rows); row counts
and the `n` sequence match the originals exactly (`n` increases by exactly 1 per row). A
row-by-row comparison against the old files shows every pre-existing key byte-identical on
`trace-1a` with zero exceptions.

> **Caveat, measured, not assumed.** On `trace-100pct`/`trace-202` 31,613 / 46,413 rows differ from
> the *old* traces — but two consecutive runs of the **same** exporter differ from each other in
> 28,601 rows as well, always starting at the same row and always in the same place:
> `Celeste/7-Summit|a-00-intro`, state `StDummy`, where the dummy-walk position alternates by whole
> pixels with bit-identical `Speed` and `movementCounter`. That is real-game nondeterminism in the
> Summit intro, not an exporter behaviour change.

Measured on `trace-202-v2.jsonl` (`--maps vendor/celeste-game/Content/Maps`):

| metric | before restoring the tail | after |
| --- | ---: | ---: |
| segments | 1,468 | 1,468 |
| `ok` | 45 | **51** |
| `mismatch` | 1,335 | 1,329 |
| `unsupported` | 87 | 87 |
| replayed frames | 36,253 | **40,989** |
| matching frames | 34,831 | **39,573** |
| `pos\|anchor=StNormal\|at=first` cluster | 41 segments / 41 frames / 0 exact | **4 segments / 4 frames / 0 exact** |
| `freezeDisagreementFrames` | 243 | 173 |
| `windDisagreementFrames` (sim ramp vs exported `Level.Wind`) | — | 64 of 40,989 |
| `duckingDisagreementFrames` | — | 319 of 40,989 |
| `geometricGroundDiffFrames` | 1,627 | 2,791 |

211 segments replayed further, 1,253 were unchanged and 4 regressed (`1-ForsakenCity|0|6c`
29→12 frames, `7-Summit|0|e-05` ×2 4→3, `4-GoldenRidge|0|c-07` 4→1 — each now dies on a different,
earlier mechanic). `Celeste/4-GoldenRidge|0|c-02` (start row 51,522), the segment that motivated
the wind work, goes from **1 replayed frame to 49** (48 exact).

`sim.rs::apply_wind_movement` also gained the source's own gate: the horizontal push is suppressed
by `Ducking && onGround` (`Player.cs:3095`) where `onGround` is the *source-private field*, which
`WindController.Update` (running before `Player.Update`) reads as the previous frame's probe — that
is `PlayerSnapshot::player_on_ground`, not the geometric `on_ground`. Using the geometric value
dropped a wind push the game applied (+70 replayed frames on the 202 trace).

### Remainder probe

`--probe-remainder N` re-anchors the first `N` mismatch segments with candidate
`PlayerSnapshot::movement_remainder` values (one axis at a time over a 1/64 grid on `[-0.5, 0.5]`,
then a joint 1/32 sweep) and reports how many frames each candidate replays before its own first
mismatch, plus that mismatch. `Monocle.Platform.movementCounter` *is* exported (`p.movementCounter`),
so the baseline is the real value; the sweep shows how sensitive a divergence is to sub-pixel state.
It is a diagnostic: the headline `exactPrefixFrames` in the report is always the unprobed replay.

### Move-surplus recovery (the analysis technique used below)

For a segment's first-mismatch frame `k`, every earlier replayed frame matched exactly, so the
simulator and the game enter frame `k` with the same integer position and the same
`movementCounter`. Therefore

```
T_sim - T_game = (Δpos_sim + mc_sim_after) - (Δpos_game + mc_game_after)
```

where `T` is the sum of the fractional `Actor.MoveH`/`MoveV` amounts applied during the frame
(`MoveVExact` deliberately does **not** touch `movementCounter`), `Δpos` is the integer position
delta of the frame and `mc_after` the final remainder. `Δpos_sim` comes from the report's
`firstMismatch.rust.pos`, `mc_sim_after` from `firstMismatch.rust.movementRemainder`, and both game
values from the trace row. A surplus that equals `amount · Δt` identifies an extra/missing
`MoveV(amount * Engine.DeltaTime)` call; an integer surplus identifies `MoveVExact`. The technique is
only as precise as the assumption that `movementCounter` did not drift earlier in the segment (a few
1e-3 in practice, since a drift large enough to change a rounding would already have failed the
`pos` comparison).

## Report schema

```jsonc
{
  "trace": "...", "mapsDir": "...", "generatedBy": "cargo run ...",
  "comparison": { "positionSource": "...", "tolerance": 0.01, "exact": [...], "unavailable": {...} },
  "notes": [...],
  "totals": { "records":N, "levelRecords":N, "nonLevelRecords":N, "parseErrors":N,
              "segments":N, "simulatedSegments":N, "skipped":N,
              "ok":N, "mismatch":N, "unsupported":N, "initError":N, "mapError":N,
              "stateMapError":N, "noLiveWindow":N, "simError":N,
              "totalFrames":N, "exactFrames":N, "stalledFrames":N, "truncated":false },
  "byUnsupportedState": { "StStarFly": { "segments":N, "frames":N, "rowsInState":N } },
  "rooms": { "<sid>|<mode>|<room>": { "segments":N, "frames":N, "ok":N, "mismatch":N,
              "unsupported":N, "other":N, "medianExactPrefix":N, "maxExactPrefix":N,
              "minExactPrefix":N } },
  "segments": [ { "sid":..., "mode":..., "room":..., "areaFile":...,
                  "startRow":N, "startFrame":N, "frames":N, "records":N,
                  "status":"ok|mismatch|unsupported|init_error|map_error|state_map_error|no_live_window|sim_error",
                  "exactPrefixFrames":N, "leadingSkippedFrames":N, "stalledFrames":N,
                  "stallOffsets":[...], "error":"...", "unsupportedState":"...",
                  "unsupportedOffset":N, "geometricGroundDiffFrames":N,
                  "freezeDisagreementFrames":N, "firstFreezeDisagreementOffset":N,
                  "firstMismatch": { "offset":N, "reasons":["pos","speed","state",...],
                                     "rust":{...}, "game":{...} },
                  "remainderProbe": {...} } ],
  "fieldMap": { "restored":[...], "derived":[...], "unrestored":[...],
                "audit": { "declaredFields":152, "restored":61, "derived":5, "unrestored":87,
                           "missing":[], "stale":[] } }
}
```

`frames` is the number of frames actually replayed (the anchor row is the initial state, so a
segment of `records` rows replays `records - leadingSkipped - stalled - 1` frames at most);
`offset` in `firstMismatch` is the row offset inside the replay window (stalled rows included).
In `byUnsupportedState`, `frames` is the sum of replayed frames of the affected segments — normally
one per segment, because `SimulationError::UnsupportedState` aborts at the first dispatch — while
`rowsInState` is the number of trace rows whose `state` name equals that state, i.e. how much of the
run those segments actually cover.

## `PlayerSnapshot` coverage

Generated by `--dump-field-map`, which enumerates `PlayerSnapshot`'s fields from its `Serialize`
implementation and checks the table against it (`missing`/`stale` are empty, otherwise the harness
prints a warning). `derived` = restored by bespoke code; `none` = no ground-truth source.

<!-- BEGIN GENERATED FIELD TABLE -->
| PlayerSnapshot field | type | trace source |
| --- | --- | --- |
| `pos` | `Vec2` | `p.Position` |
| `speed` | `Vec2` | `p.Speed` |
| `movement_remainder` | `Vec2` | `p.movementCounter` |
| `current_lift_speed` | `Vec2` | `p.currentLiftSpeed` |
| `last_lift_speed` | `Vec2` | `p.lastLiftSpeed` |
| `lift_speed_timer` | `f32` | `p.liftSpeedTimer` |
| `ignore_jump_thrus` | `bool` | `p.IgnoreJumpThrus` |
| `dashes` | `u8` | `p.Dashes` |
| `stamina` | `f32` | `p.Stamina` |
| `on_ground` | `bool` | `p.onGround` |
| `player_on_ground` | `bool` | `p.onGround` |
| `dead` | `bool` | `p.<Dead>k__BackingField` |
| `just_respawned` | `bool` | `p.JustRespawned` |
| `dash_dir` | `Vec2` | `p.DashDir` |
| `last_aim` | `Vec2` | `p.lastAim` |
| `before_dash_speed` | `Vec2` | `p.beforeDashSpeed` |
| `demo_dashed` | `bool` | `p.demoDashed` |
| `dash_started_on_ground` | `bool` | `p.dashStartedOnGround` |
| `dash_attack_timer` | `f32` | `p.dashAttackTimer` |
| `dash_cooldown_timer` | `f32` | `p.dashCooldownTimer` |
| `dash_refill_cooldown_timer` | `f32` | `p.dashRefillCooldownTimer` |
| `boost_target` | `Vec2` | `p.boostTarget` |
| `boost_red` | `bool` | `p.boostRed` |
| `no_wind_timer` | `f32` | `p.noWindTimer` |
| `wall_slide_timer` | `f32` | `p.wallSlideTimer` |
| `wall_slide_dir` | `i8` | `p.wallSlideDir` |
| `jump_grace_timer` | `f32` | `p.jumpGraceTimer` |
| `auto_jump` | `bool` | `p.AutoJump` |
| `auto_jump_timer` | `f32` | `p.AutoJumpTimer` |
| `var_jump_timer` | `f32` | `p.varJumpTimer` |
| `var_jump_speed` | `f32` | `p.varJumpSpeed` |
| `max_fall` | `f32` | `p.maxFall` |
| `move_x` | `i8` | `p.moveX` |
| `force_move_x` | `i8` | `p.forceMoveX` |
| `force_move_x_timer` | `f32` | `p.forceMoveXTimer` |
| `wall_speed_retention_timer` | `f32` | `p.wallSpeedRetentionTimer` |
| `wall_speed_retained` | `f32` | `p.wallSpeedRetained` |
| `wall_boost_timer` | `f32` | `p.wallBoostTimer` |
| `wall_boost_dir` | `i8` | `p.wallBoostDir` |
| `hop_wait_x` | `i8` | `p.hopWaitX` |
| `hop_wait_x_speed` | `f32` | `p.hopWaitXSpeed` |
| `min_hold_timer` | `f32` | `p.minHoldTimer` |
| `climb_no_move_timer` | `f32` | `p.climbNoMoveTimer` |
| `last_climb_move` | `i8` | `p.lastClimbMove` |
| `dream_dash_can_end_timer` | `f32` | `p.dreamDashCanEndTimer` |
| `launch_approach_x` | `Option<f32>` | `p.launchApproachX` |
| `summit_launch_target_x` | `f32` | `p.summitLaunchTargetX` |
| `summit_launch_particle_timer` | `f32` | `p.summitLaunchParticleTimer` |
| `star_fly_timer` | `f32` | `p.starFlyTimer` |
| `star_fly_transforming` | `bool` | `p.starFlyTransforming` |
| `star_fly_speed_lerp` | `f32` | `p.starFlySpeedLerp` |
| `star_fly_last_dir` | `Vec2` | `p.starFlyLastDir` |
| `strawberry_collect_index` | `u16` | `p.StrawberryCollectIndex` |
| `strawberry_collect_reset_timer` | `f32` | `p.StrawberryCollectResetTimer` |
| `explode_launch_boost_timer` | `f32` | `p.explodeLaunchBoostTimer` |
| `explode_launch_boost_speed` | `f32` | `p.explodeLaunchBoostSpeed` |
| `dummy_moving` | `bool` | `p.DummyMoving` |
| `dummy_gravity` | `bool` | `p.DummyGravity` |
| `dummy_friction` | `bool` | `p.DummyFriction` |
| `dummy_maxspeed` | `bool` | `p.DummyMaxspeed` |
| `launched` | `bool` | `p.launched` |
| `state` | derived | top-level `state` name (`PlayerStates.GetCurrentStateName`) mapped exhaustively onto `PlayerState`. |
| `facing` | derived | `p.Facing` (`Facings` enum: -1 Left, 1 Right) compared against the boolean `facing`. |
| `time_rate` | derived | top-level `timeRate` (`Engine.TimeRate`). |
| `player_on_ground_initialized` | derived | set to true; the anchor row is a post-`Player.Update` capture, so the source-private `onGround` is authoritative. |
| `frame_delta_time` | derived | `#[serde(skip)]` on the wire type. `Simulator::step` recomputes it every frame as the supplied `rawDt` bits times `time_rate`. |
| `wind` | derived | top-level `wind` (`Celeste.Level.Wind`, `Level.cs:149`). Restored verbatim from the anchor row so the room segment continues the source's own ramp: `WindController.Update` rewrites it with `Calc.Approach(level.Wind, targetSpeed, 1000f * Engine.DeltaTime)` (`WindController.cs:194`) and displaces the Player's `WindMover` component (`Player.cs:1180`) by `level.Wind * 0.1f * Engine.DeltaTime` (`WindController.cs:199-201`). |
| `wind_target` | derived | top-level `windTarget` (`WindController.targetSpeed`, `WindController.cs:47`, read by reflection through the private `Level.windController` field at `Level.cs:101`). The replayed ramp needs the source's own target, which a room segment cannot reconstruct. |
| `ducking` | derived | top-level `ducking` (`Player.Ducking`, `Player.cs:1005-1028`). A computed property over `Monocle.Entity.Collider` (`Monocle/Entity.cs:73`), so it is not a declared field; the exporter also writes the active collider as `collider` = `[absoluteLeft, absoluteTop, width, height]` (`Monocle/Collider.cs:229,205,12,14`). |
| `freeze_timer` | derived | top-level `freezeTimer` (`Monocle.Engine.FreezeTimer`, `Monocle/Engine.cs:28`). While positive `Engine.Update` only decrements it and skips `Scene.Update` entirely (`Engine.cs:266-269`); the exporter writes the post-decrement value, which is exactly the snapshot state `Simulator::step` reads at the top of the next frame. |
| `can_dream_dash` | derived | top-level `inventory.DreamDash` (`Celeste.Session.Inventory`, `Session.cs:35`, `PlayerInventory.cs:24`). `Player.Inventory` forwards it (`Player.cs:956-966`) and the source reads it at `Player.cs:3420` and `4500`; restoring it removes the need to infer the flag from `dreamDashCanEndTimer`. |
| `badeline_boost_active` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `badeline_boost_collidable` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `badeline_boost_current_position` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `badeline_boost_entity_origin` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `badeline_boost_final` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `badeline_boost_frame` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `badeline_boost_phase` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `badeline_boost_relocating` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `badeline_boost_relocation_duration` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `badeline_boost_relocation_elapsed` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `badeline_boost_relocation_from` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `badeline_boost_relocation_to` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `badeline_boost_stage` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `badeline_boost_start` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `badeline_boost_target` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `booster_boosting` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `booster_reuse_timer` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `bounce_blocks` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `bounce_reuse_timer` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `bumper_reuse_timer` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `bumpers` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `camera` | none | not reachable from the player's base chain (`Celeste.Player` -> `Monocle.Actor` -> `Monocle.Platform` -> `Monocle.Entity`), so the reflection dump cannot see it |
| `camera_initialized` | none | not reachable from the player's base chain (`Celeste.Player` -> `Monocle.Actor` -> `Monocle.Platform` -> `Monocle.Entity`), so the reflection dump cannot see it |
| `carried_strawberries` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `cassette_blocks` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `cassette_manager` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `clouds` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `core_mode` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `crouch_dash_buffer_timer` | none | `VirtualButton` buffers live on the static `Celeste.Input` object, not on `Player`; `Simulator::step` rebuilds them from the press edges |
| `current_room_bounds` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `dash_buffer_timer` | none | `VirtualButton` buffers live on the static `Celeste.Input` object, not on `Player`; `Simulator::step` rebuilds them from the press edges |
| `dash_end_pending` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `death_freeze_pending` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `exit_blocks` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `falling_blocks` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `feather_reuse_timer` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `gliders` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `heart_gems` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `holding_glider` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `holding_theo` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `invisible_barriers` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `jump_buffer_timer` | none | `VirtualButton` buffers live on the static `Celeste.Input` object, not on `Player`; `Simulator::step` rebuilds them from the press edges |
| `killboxes` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `last_badeline_boost_target` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `last_booster_target` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `last_bounce_target` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `last_bumper_target` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `last_feather_target` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `lookouts` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `move_blocks` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `moving_solid_time` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `neutral_wall_jump_friction_delay` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `pending_bounce_from_y` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `pickup_old_speed` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `pickup_old_var_jump_timer` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `pickup_timer` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `post_transition_normal_updates` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `refills` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `reflection_fall_frames` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `reflection_fall_phase` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `reflection_fall_wait_timer` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `respawn_frames` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `rising_lavas` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `sandwich_lavas` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `scene_time_active` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `seekers` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `spinners` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `star_fly_hitbox_preserved` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `star_fly_transform_frames` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `state_timer` | none | `Player.StateMachine` is a `StateMachine` object; the exporter skips non-primitive fields, so `StateMachine.Timer` is absent |
| `strawberry_collect_timer` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `strawberry_follow_delay_timer` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `strawberry_picked_mask` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `temple_fall_landed` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `temple_fall_wait_frames` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `temple_gates` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `theo_crystals` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |
| `transition_direction` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `transition_room_bounds` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `transition_target` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `transition_timer` | none | no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*) |
| `zip_movers` | none | room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room |

Declared `PlayerSnapshot` fields: 152. Restored from a `p` key: 61. Derived: 10. Unrestored: 82. Missing from table: []. Stale table entries: [].
<!-- END GENERATED FIELD TABLE -->

Notes on the table:

* `player_on_ground` shares its source with `on_ground` on purpose (both mirror `p.onGround`; see
  [above](#what-is-compared-and-what-cannot-be)).
* `p.launchApproachX` is a nullable C# field: the exporter skips `null` values, so the key is absent
  until the player enters a launch state. Absent keys leave the field at its `Default`.
* `ducking`, `state_timer`, `camera`, `freeze_timer` and the `VirtualButton` buffers are the only
  *comparison-relevant* gaps; every other `none` is simulator-internal bookkeeping or room-entity
  state that `Simulator::new` initialises from the decoded room.

## Findings — `trace-1a.jsonl` (Forsaken City A-side, 3,215 rows)

Real output with `celeste-physics` at the JumpThru fix (`5299096`):

```
$ cargo run -q -p celeste-physics --release --example tas_fidelity -- \
    --trace .tmp/tasrun/trace-1a.jsonl --maps vendor/celeste-game/Content/Maps \
    --out .tmp/tasrun/report-1a.json
segments=20 simulated=20 ok=1 mismatch=18 unsupported=1 init_error=0 map_error=0 \
state_map_error=0 no_live_window=0 sim_error=0 skipped=0 frames=741 exact=722 stalled=229
```

| metric | value | before the JumpThru fix |
| --- | ---: | ---: |
| rows read / Level rows / non-level | 3,215 / 3,213 / 2 | same |
| segments (all simulated) | 20 | same |
| status: `ok` / `mismatch` / `unsupported` | 1 / 18 / 1 | same |
| replayed frames | **741** | 567 |
| matching frames (`exactFrames`) | **722 (97.4% of replayed frames)** | 548 (96.6%) |
| rows where `Player.Update` did not run | 229 | same |
| `freezeDisagreementFrames` (all segments) | 0 | 0 |
| parse errors, map errors, state-map errors, panics | 0 | 0 |

Room `3` replays **106/106 frames exactly**. The single `unsupported` segment is the level's first
room, which starts in `StIntroJump` — a state the source-informed subset does not implement
(`Simulator::step` returns `SimulationError::UnsupportedState`).

### Worst rooms (smallest `exactPrefixFrames`)

Ranked by median `exactPrefixFrames`, then by replayed frames. Generated from the report.

<!-- BEGIN WORST ROOMS -->
| room (`sid\|mode\|room`) | replayed frames | median exactPrefix | first differing field(s) |
| --- | ---: | ---: | --- |
| `Celeste/1-ForsakenCity|0|1` | 1 | 0 | `unsupported` |
| `Celeste/1-ForsakenCity|0|12a` | 2 | 1 | `pos, speed, state` |
| `Celeste/1-ForsakenCity|0|11` | 5 | 4 | `pos, speed` |
| `Celeste/1-ForsakenCity|0|6c` | 15 | 14 | `pos, speed` |
| `Celeste/1-ForsakenCity|0|6` | 22 | 21 | `pos, speed` |
| `Celeste/1-ForsakenCity|0|9` | 22 | 21 | `pos, speed, stamina, state` |
| `Celeste/1-ForsakenCity|0|8` | 25 | 24 | `pos` |
| `Celeste/1-ForsakenCity|0|4` | 27 | 26 | `pos, speed` |
| `Celeste/1-ForsakenCity|0|3b` | 29 | 28 | `pos` |
| `Celeste/1-ForsakenCity|0|9b` | 30 | 29 | `speed, stamina` |
| `Celeste/1-ForsakenCity|0|12` | 34 | 33 | `pos` |
| `Celeste/1-ForsakenCity|0|7` | 34 | 33 | `pos` |
<!-- END WORST ROOMS -->

### The JumpThru collider bug (found here, fixed in `5299096`)

Before the fix, rooms `7`, `10a`, `8`, `3b`, `5`, `6`, `6b`, `6c` and `12` all diverged **on their
first replayed frame** with `reasons: ["pos"]` — a 1 px difference in `pos.y` (the simulator one
pixel higher) while `speed`, `state`, `stamina`, `facing`, `dashes` and `on_ground` all matched.

Two trace rows pinned the mechanism exactly:

```
n=1641 room=7  Position=[2599,-1464] movementCounter.y=0.3333282470703125  Speed=[170,-160]   (anchor)
n=1642 room=7  Position=[2602,-1466] movementCounter.y=-0.333343505859375  Speed=[165.66665649414062,-160]
sim after that frame:                Position=[2602,-1467] movementCounter.y=-0.000011444091796875
```

The recovered per-frame `MoveV` totals (see [move-surplus recovery](#move-surplus-recovery-the-analysis-technique-used-below))
were `T_game = -2.666 66` (= `Speed.Y · Δt`) and `T_sim = -3.333 34`, a difference of exactly
**`-40 · Δt`**: the **JumpThru Assist** of `Player.Update` (`Player.cs` L1787-1790), which the
simulator applied where the game did not.

```csharp
if (!onGround && Speed.Y <= 0f && (StateMachine.State != 1 || lastClimbMove == -1)
    && CollideCheck<JumpThru>() && !JumpThruBoostBlockedCheck())
{
    MoveV(-40f * Engine.DeltaTime);
}
```

The geometry decided it by three pixels: the player hitbox (`Player.normalHitbox`, `Player.cs` L605
= `Hitbox(8,11,-4,-11)`) occupied `y ∈ [-1475,-1464]`, the room's `jumpThru` was decoded as
`bounds = (2592,-1480,24,8)` → `y ∈ [-1480,-1472]` (overlap), while the real collider is
`JumpThru.cs`'s `base.Collider = new Hitbox(width, 5f)` → `y ∈ [-1480,-1475]`, which merely touches
the player's head and therefore does not collide (`Collider.Collide` uses strict inequalities).
The decoded 8 is the **entity's** `.bin` `height`; `Celeste.JumpThru` overrides its collider to 5.
`celeste-physics` now decodes `jumpThru` colliders through
`entity_decode::JUMP_THRU_COLLIDER_HEIGHT = 5.0` (commit `5299096`).

Measured effect (same harness, same trace): `frames=567 exact=548` → **`frames=741 exact=722`**;
per-room replayed frames `7: 1 → 34`, `8: 3 → 25`, `10a: 1 → 76`, `6b: 27 → 71`, all other rooms
unchanged.

### The next divergence class

With the JumpThru fix in place, no remaining 1A mismatch dies on its first frame for a `-40·Δt`
reason any more. Recovering the surplus for all 18 mismatches gives:

| room | row | first differing field(s) | `T_sim − T_game` (x) | (y) | reading |
| --- | ---: | --- | ---: | ---: | --- |
| `3b` | 686 | `pos` | +0.111191 | 0 | one extra/missing small x move (≈ 6.7·Δt) |
| `5` | 832 | `pos` | 0 | **−0.916707** | game applied a fractional y move (≈ −55·Δt worth), simulator none |
| `7` | 1675 | `pos` | +0.327863 | 0 | ≈ +20·Δt of x movement |
| `8` | 1809 | `pos` | 0 | +0.499957 | ≈ +30·Δt of y |
| `6b` | 1409 | `pos` | 0 | +1.333336 | exactly `+80·Δt` (`Calc.Approach(ExactPosition, …, 80f·Δt)` boost move) |
| `8b` | 2005 | `pos` | 0 | +2.000000 | exactly `+120·Δt` (or `MoveVExact(2)`) |
| `10a` | 2396 | `pos` | 0 | +5.583207 | simulator `T_y = -0.666 668` = **`-40·Δt` again**; game `T_y = -6.25` |
| `12` | 2799 | `pos` | −0.083294 | 0 | ≈ −5·Δt |
| `4` | 553 | `pos`, `speed` | +4.999995 | +1.000000 | state divergence (sim `speed=(0,0)` vs game `(0,-240)`, both `StDash`) |
| `6c` | 1503 | `pos`, `speed` | 0 | −0.166672 | ≈ −10·Δt plus a `speed.y` divergence (20 vs 30) |
| `11` | 2569 | `pos`, `speed` | +2.091672 | 0 | wall jump: game `speed.x=-136.33`, sim `-10.83` |
| `12a` | 2954 | `pos`, `speed`, `state` | −1.514385 | +1.419123 | mid-dash anchor, `StateMachine.Timer` unrestorable |
| `2`, `9`, `end` | 265 / 2092 / 3171 | `pos`, `speed`, `stamina`, `state` | 0 | −1.078429 | dash-vs-wall-jump state sequence |

What this pins down, and what it rules out:

* **`JumpThruBoostBlockedCheck()` is not the cause of any 1A residual.** No mismatch has a `-40·Δt`
  surplus at the segment's first frame, and in the one case that still shows a `-40·Δt` surplus
  (room `10a`, row 2396) a `LedgeBlocker` cannot explain it: room `10a`'s only `LedgeBlocker` owners
  are `Spikes` at `x ∈ {3848, 3864, 3880, 3896}`, which is ≥ 60 px from the player's box at
  `Position − UnitY*2`, and the `jumpThru` at `(3960,-2336,16,5)` still overlaps **both** the 11 px
  `normalHitbox` and the 6 px `duckHitbox` (player `Position = (3961,-2336)`). So the game must be
  skipping the assist for a different reason — most plausibly that `Player.DashUpdate`'s own
  `MoveVExact` corner correction runs inside the state callback, before `Player.Update`'s assist
  block, so the game evaluates `CollideCheck<JumpThru>()` at a different position than the
  simulator does. The `LedgeBlocker` gap flagged earlier remains open but is **latent**: not
  observable in Forsaken City 1A.
* **`Player.Ducking` is the highest-value missing field.** Rooms `6`, `8` and `10a` all have
  `ducking`/`wasDucking` true in the trace around their mismatch (crouch-dash/down-dash entry), and
  `ducking` is not exported, so the anchored simulator can hold the 11 px `normalHitbox` where the
  game has the 6 px `duckHitbox`. Every collider probe (`CollideCheck<JumpThru>`,
  `CollideCheck<Solid>`, the ceiling push, `CanUnDuck`) then reads a different box. Deriving
  `ducking` from `wasDucking` (which lags `Ducking` by one frame when it changes) plus the `Ducking`
  setter rules is the cheapest next step; exporting the active collider would be exact.
* **`StateMachine.Timer` is the second.** `12a` anchors mid-`StDash`; the simulator's `state_timer`
  starts at 0 and `dash_update` ends the dash immediately, so its state becomes `StNormal` at
  `EndDashSpeed` (`113.14, -84.85` = `160·(cos45, -sin45)`) while the game is still dashing at
  `(204, -170)`. `p.dashAttackTimer` (`DashAttackTime = 0.3` at `DashBegin`, decremented in
  `Player.Update`) is an exported proxy for the dash's age, so `state_timer` is derivable rather
  than guessable.
* Everything else is a genuine physics/state divergence of the kind the tech handbook tracks: wall
  jump horizontal speed (`11`, and `2`/`9`/`end`, where the simulator ends a diagonal dash while the
  game wall jumps with `stamina` `110 → 82.5`), the falling-speed branch of `Player.NormalUpdate`
  (`6a`: `speed.y` `165.000015` game vs `184.705658` sim with identical `pos`), and a one-frame
  `speed`-only difference (`6c`).

## Findings — full-game traces

Both whole-game traces were replayed end to end with the same binary. `trace-202.jsonl` is a
superset of `trace-100pct.jsonl`: it adds the berry-collection detours of the 202-berry TAS.

| metric | `trace-100pct.jsonl` | `trace-202.jsonl` |
| --- | ---: | ---: |
| JSONL rows | 281,113 | 461,122 |
| Level rows | 266,262 | 438,303 |
| non-level rows | 14,851 | 22,819 |
| segments (all simulated) | 918 | 1,468 |
| status `ok` | 30 | 45 |
| status `mismatch` | 840 | 1,335 |
| status `unsupported` | 48 | 87 |
| status `no_live_window` | 0 | 1 |
| init / map / state-map / sim errors | 0 / 0 / 0 / 0 | 0 / 0 / 0 / 0 |
| replayed frames | 21,653 | 36,253 |
| matching frames (`exactFrames`) | **20,765 (95.9%)** | **34,831 (96.1%)** |
| rows where `Player.Update` did not run | 28,458 | 45,319 |
| `freezeDisagreementFrames` | 181 | 243 |
| `geometricGroundDiffFrames` | 970 | 1,637 |
| parse errors / panics | 0 / 0 | 0 / 0 |

The `unsupported` segments are story-intro states the source-informed subset deliberately omits
(`byUnsupportedState`; `rowsInState` is how many trace rows those segments cover):

| state | 100% segments | 100% rows in state | 202 segments |
| --- | ---: | ---: | ---: |
| `StIntroJump` | 23 | 1,947 | 45 |
| `StIntroWalk` | 17 | 1,083 | 29 |
| `StIntroWakeUp` | 7 | 1,234 | 12 |
| `StIntroThinkForABit` | 1 | 97 | 1 |
| other states | 0 | 0 | 0 |

First-mismatch reason histogram (a segment counts once per listed reason):

| reason | 100% | 202 |
| --- | ---: | ---: |
| `pos` | 584 | 931 |
| `speed` | 514 | 822 |
| `state` | 254 | 397 |
| `stamina` | 98 | 161 |
| `dashes` | 70 | 104 |
| `on_ground` | 36 | 56 |
| `dead` | 10 | 19 |
| `facing` | 2 | 3 |

### Worst rooms, 100% run (smallest `exactPrefixFrames`)

<!-- BEGIN WORST ROOMS 100 -->
| room (`sid\|mode\|room`) | replayed frames | median exactPrefix | first differing field(s) |
| --- | ---: | ---: | --- |
| `Celeste/7-Summit|0|f-00` | 356 | 0 | `unsupported` |
| `Celeste/5-MirrorTemple|0|b-00` | 79 | 0 | `state` |
| `Celeste/2-OldSite|0|start` | 23 | 0 | `unsupported` |
| `Celeste/4-GoldenRidge|0|b-sec` | 4 | 0 | `pos, speed, state` |
| `Celeste/5-MirrorTemple|0|c-00` | 2 | 0 | `unsupported` |
| `Celeste/5-MirrorTemple|1|c-00` | 2 | 0 | `unsupported` |
| `Celeste/7-Summit|0|g-00b` | 2 | 0 | `unsupported` |
| `Celeste/7-Summit|1|e-00` | 2 | 0 | `unsupported` |
| `Celeste/8-Epilogue|0|inside` | 2 | 0 | `pos, speed, state` |
| `Celeste/8-Epilogue|0|outside` | 2 | 0 | `unsupported` |
| `Celeste/0-Intro|0|0` | 1 | 0 | `unsupported` |
| `Celeste/1-ForsakenCity|0|1` | 1 | 0 | `unsupported` |
<!-- END WORST ROOMS 100 -->

Rooms that replay furthest (context; 100% run):

| room | replayed frames | median `exactPrefix` |
| --- | ---: | ---: |
| `Celeste/5-MirrorTemple\|0\|void` | 855 | 854 |
| `Celeste/LostLevels\|0\|end-cinematic` | 710 | 710 (`ok`) |
| `Celeste/LostLevels\|0\|end-granny` | 240 | 240 (`ok`) |
| `Celeste/9-Core\|1\|b-00` | 211 | 210 |
| `Celeste/6-Reflection\|0\|b-02b` | 209 | 208 |
| `Celeste/5-MirrorTemple\|0\|b-02` | 178 | 23 |

A room key can hold several segments: `7-Summit|0|f-00` has a 354-frame `ok` segment plus two
1-frame segments, so its *median* prefix is 0. Rank by `maxExactPrefix` when the question is "how far
did this room ever get".

The single `no_live_window` segment is 19 consecutive `Celeste/LostLevels|0|end-granny` rows where
`scene == "Level"` but the row carries no `state`/`p` at all — the exporter found no `Player` entity
(`level.Tracker.GetEntity<Player>()` returned null), so there is no snapshot to anchor on. The
anchor search skips such rows, which is why the same trace produces no `state_map_error` at all.

### Worst rooms, 202-berry run

<!-- BEGIN WORST ROOMS 202 -->
| room (`sid\|mode\|room`) | replayed frames | median exactPrefix | first differing field(s) |
| --- | ---: | ---: | --- |
| `Celeste/7-Summit|0|f-00` | 356 | 0 | `unsupported` |
| `Celeste/2-OldSite|0|start` | 24 | 0 | `unsupported` |
| `Celeste/3-CelestialResort|0|13-a` | 21 | 0 | `pos, speed` |
| `Celeste/4-GoldenRidge|0|b-sec` | 10 | 0 | `pos` |
| `Celeste/4-GoldenRidge|0|c-00` | 7 | 0 | `pos` |
| `Celeste/5-MirrorTemple|0|c-00` | 3 | 0 | `unsupported` |
| `Celeste/5-MirrorTemple|1|c-00` | 3 | 0 | `unsupported` |
| `Celeste/7-Summit|0|g-00b` | 3 | 0 | `unsupported` |
| `Celeste/7-Summit|1|e-00` | 3 | 0 | `unsupported` |
| `Celeste/1-ForsakenCity|0|1` | 2 | 0 | `unsupported` |
| `Celeste/1-ForsakenCity|1|00` | 2 | 0 | `unsupported` |
| `Celeste/1-ForsakenCity|1|04` | 2 | 0 | `pos, speed, stamina, state` |
<!-- END WORST ROOMS 202 -->

## Diagnosis for the workstreams

Two classes were recovered with the technique above, from `report-202.json` +
`trace-202.jsonl`. The analysis script is a scratch tool
(`.tmp/tasrun/move-totals.mjs`), not part of the repo.

### D1 — "pure 1 px `pos`" on the first replayed frame: a constant `40 * Δt` x offset

The requested example, `Celeste/4-GoldenRidge|0|c-02` (startRow 51522, anchorRow 51562,
mismatch row 51563, `reasons: ["pos"]`, `state = StNormal`, speeds bit-identical
`(304.33331298828125, 22.500043869018555)`):

| quantity | value |
| --- | --- |
| anchor `Position` / `movementCounter` | `(6312, -2317)` / `(0.4777865409851074, 0.2500009834766388)` |
| game row 51563 | `Position (6317, -2316)`, `movementCounter (-0.11664962768554688, -0.3749975562095642)` |
| simulator after that frame | `Position (6318, -2316)`, `movementCounter (-0.449981689453125, -0.3749975562095642)` |
| recovered `T_game` / `T_sim` | `x: 4.405564 / 5.072232`, `y: 0.375001 / 0.375001` |
| **surplus** | **`x: +0.666668 = +40·Δt`**, `y: 0` |

So the two sides agreed on every engine quantity except that the simulator's horizontal MoveH
total is exactly `40·Δt` larger: the simulator moved `Speed.X · Δt = 304.3333 · Δt` while the game's
frame moved `264.3338 · Δt`, and the trace's *reported* `Speed.X` is the same 304.3333 on both
sides. Rows 51562–51566 show the same constant offset (implied move-time `Speed.X` = reported
`Speed.X` − 40, every frame), with `wallSpeedRetained = 365`, `wallSpeedRetentionTimer = 0`,
`forceMoveX = 1`, `DashDir = (0.7071, 0.7071)`, `Stamina = 82.5`.

Two readings fit the arithmetic exactly, and telling them apart needs a sim-side instrument:

* **ordering**: the game's `MoveH(Speed.X * Δt)` (`Player.cs` L1801) ran with `Speed.X` *before* a
  `+40` x write that lands later in the frame, while the simulator applies the `+40` before its move.
  The only `Speed.X += 40` in `Player.cs` is `L2446` inside `Player.Jump()`
  (`Speed.X += 40f * (float)moveX`), but the trace **rules that writer out for this frame**:
  `Jump()` also sets `varJumpTimer = 0.2f` and `varJumpSpeed = Speed.Y = -105f`, and row 51563 has
  `varJumpTimer = 0`, `varJumpSpeed = -105` (stale) and `Speed.Y = 22.5`. So if this reading is
  right, the writer is a path that adds 40 px/s to `Speed.X` without a jump (candidates:
  `Level.EnforceBounds(player)` at `Player.cs` L1917, which runs *after* both moves and can clamp
  and rewrite `Speed`; the `onCollideH` callback; or a `StaticMover`/lift carry).
* **an extra move**: the game applied `MoveH(+40·Δt)` on top of its main move, i.e. a horizontal
  `MoveH(40f * Engine.DeltaTime)`-shaped call the simulator lacks. `Player.cs`'s fractional MoveH
  sites are only L1801, L3628/L3633 (`±50·Δt`) and L6164 (`32·Δt`), so this would have to come from a
  carried/moving solid or a `WindMove`-style displacement rather than a direct Player call.

Both readings point at the same place: a `40·Δt`-shaped horizontal displacement tied to
`wallSpeedRetained`/wall-jump trajectories (W2) and to the post-move `EnforceBounds` window (W6).
193 of the 1,335 mismatch segments diverge on their very first replayed frame and are recoverable
this way; of those, 55 are `pos`-only with bit-identical speeds, bucketed by surplus as
`40·Δt` (8), `80·Δt` (10, matches the `80f·Δt` booster approach), `100·Δt`, `160·Δt`, `30·Δt`,
`-15·Δt`, `-20·Δt`, `-105·Δt`, `-52.5·Δt`, `120·Δt` … plus 48 with a zero surplus (the totals agree,
so those agree on the float sum and differ only in rounding/splitting) and 6 with an integer surplus
(`MoveHExact`/`MoveVExact`).

### D2 — `on_ground` disagreement: not a harness artifact

The trace's `p.onGround` is `Celeste.Player.onGround`, a private field written **only** in
`Player.Update` (`Player.cs` L1499-1526) and never rewritten later in the frame — not by entities,
not by `EnforceBounds`. It is a *start-of-frame* probe: at that point the frame has not moved the
player yet (the move pass is L1799-1806) and `Speed.Y` is still the previous frame's value, and the
probe itself is

```csharp
if (StateMachine.State == 9) onGround = false;                       // StDreamDash
else if (Speed.Y >= 0f) {
    Platform platform = CollideFirst<Solid>(Position + Vector2.UnitY);
    if (platform == null) platform = CollideFirstOutside<JumpThru>(Position + Vector2.UnitY);
    onGround = platform != null;  OnSafeGround = platform?.Safe ?? false;
} else onGround = false;                                             // rising
```

`PlayerSnapshot::player_on_ground` is the simulator's mirror of exactly that quantity
(`sim.rs` L4807-4812: `state != DreamDash && speed.y >= 0.0 && grounded(p, map)`, evaluated before
the state dispatch, i.e. also start-of-frame), which is why the harness compares the trace's
`onGround` against it. `PlayerSnapshot::on_ground` is a *different* quantity by construction — the
post-entity geometric re-probe (`sim.rs` L4867) — and comparing the trace against it produces false
mismatches, which is why the harness does not.

So the 36 `on_ground`-reason segments (100% run; 63 in 202) are **not** a gate-restore bug in the
harness: the anchor restores both `player_on_ground` and `on_ground` from `p.onGround` and marks
`player_on_ground_initialized = true`, and the comparison is like-for-like. They are a
`grounded(p, map)` vs `CollideFirst<Solid>(Position + UnitY)` / `CollideFirstOutside<JumpThru>`
divergence. Worth checking, in order: (a) whether `grounded()` honours `Entity.Collidable == false`
and the `Outside` semantics of `CollideFirstOutside<JumpThru>`; (b) whether it probes
`Position + UnitY` (a 1 px shift of the *collider*) or a hand-rolled rect; (c) whether the
`Speed.Y >= 0f` gate reads the same `Speed.Y` the game had at that instant (the simulator sets
`player_on_ground` before its state dispatch, but the game's probe runs before `base.Update()`,
i.e. before the state callback — if the simulator's `NormalUpdate` has already touched `speed.y` by
then, the gate can disagree). `geometricGroundDiffFrames` (970 frames in 100%, 1,637 in 202) counts
where the simulator's own two ground values disagree, which is the fastest way to find the frames
worth instrumenting.

## Reproduce

```powershell
cd D:\celeste-research\celeste-next-gym

# build (use a private target dir if another session holds the shared one)
$env:CARGO_TARGET_DIR = 'D:\celeste-research\.tmp\tasrun\target-fidelity'
cargo build -p celeste-physics --release --example tas_fidelity

# 1A (fast iteration trace)
cargo run -q -p celeste-physics --release --example tas_fidelity -- `
  --trace D:\celeste-research\.tmp\tasrun\trace-1a.jsonl `
  --maps vendor\celeste-game\Content\Maps `
  --out D:\celeste-research\.tmp\tasrun\report-1a.json

# optional diagnostics + Markdown
cargo run -q -p celeste-physics --release --example tas_fidelity -- `
  --trace D:\celeste-research\.tmp\tasrun\trace-1a.jsonl `
  --maps vendor\celeste-game\Content\Maps `
  --out D:\celeste-research\.tmp\tasrun\report-1a-probe.json --probe-remainder 8
node tools\tas-fidelity\render-report.mjs .tmp\tasrun\report-1a.json .tmp\tasrun\report-1a.md

# whole-game traces (~7 min for 100%, ~12 min for 202 on this machine)
cargo run -q -p celeste-physics --release --example tas_fidelity -- `
  --trace D:\celeste-research\.tmp\tasrun\trace-100pct.jsonl `
  --maps vendor\celeste-game\Content\Maps `
  --out D:\celeste-research\.tmp\tasrun\report-100pct.json
```

## Known limitations

* Room-transition rows (`leadingSkippedFrames`) cannot be replayed: the transition coroutine state
  is not part of the exported field chain, so a segment always starts after the transition
  completes. This under-counts the frames a real run would cover by ~40 rows per room change, and it
  means the harness never checks `Player.TransitionTo`'s 60 px/s slide or the `Player.OnTransition`
  dash/stamina refill in flight (the refill is captured by the post-transition anchor snapshot).
* `ducking`, `StateMachine.Timer` and `Level.Camera.Position` are unobservable from the trace; a
  segment anchored while any of them is mid-flight will diverge for that reason rather than for a
  physics reason. The report always names the first differing *compared* field, so read the
  mechanism column with that in mind.
* Entity runtime state (`zip_movers`, `falling_blocks`, `bumpers`, `seekers`, …) is re-initialised
  from the decoded room at the anchor, because the trace only exports the player field chain.
  Segments that begin while a mover is mid-cycle will diverge.
* Only `--rooms`/`--min-frames` filtering and per-segment caps exist; there is no way to re-anchor a
  segment in the middle of a room (that is what a "sub-segment" mode would do, and it would hide
  exactly the divergences this report exists to find).
