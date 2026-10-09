# TAS replay fidelity environment (vanilla 202 / 100%)

Objective: run the **vanilla Celeste full-game TAS** as a fidelity gate for `celeste-physics`
("next-gym"). The real game produces the ground truth; next-gym must reproduce it frame by frame.

## Status and handoff

**Where it stands.** The environment works end to end and the gate is real: the real game plays the
pinned vanilla 202-berry TAS, the instrumented CelesteTAS dumps one record per executed frame, and
`tas_fidelity` replays every room segment through `Simulator` and stops at its first divergence.

| trace | `ok` rooms | mismatch | unsupported | replayed frames | frame-exact |
| --- | ---: | ---: | ---: | ---: | ---: |
| `trace-202-v5` | **503** | 964 | 0 | **157,982** | **156,987** |
| `trace-100pct-v5` | **329** | 589 | 0 | **94,728** | **94,122** |
| `trace-1a-v5` | **16** | 4 | 0 | **2,129** | **2,125** |

The latest step is **CrumblePlatform** (`0fc9e6d`): the floor under the divergence above is a `crumbleBlock`, and with its collapse sequence modelled (`CrumblePlatform.cs:94-169`) the corpus is clean where a plain solid measured six regressions: **202 38 improved / 1430 identical / 0 regressed**, `484 -> 502` `ok`, `+4,018` frames; **100pct 23 / 895 / 0**, `318 -> 328`, `+2,228`; `1a` unchanged. The coroutine state lives on `Simulator` (cloned by `fork`, invisible to the field-map audit), collapse uses the parked-bounds idiom, and the state had to be threaded through the free `step` because `advance_post_player_entities` is reached from three branches - the first attempt wired two of them and ran the coroutine twice a frame.

The step before that was the **landing frame's vertical move** (`b94ab72`): the landing branch of
`INTRO_PHASE_JUMP_FALL` zeroed `Speed.Y` before the frame's physics, so the simulator skipped that
frame's move. The source's coroutine only stops *adding* gravity once `onGround` is true, and the
accumulated fall speed still drives `MoveV(Speed.Y * dt)` into the floor - the blocked step zeroes
`movementCounter.Y`, and `Player.OnCollideV` zeroes `Speed.Y`. Measured on
`7-Summit|1|g-00|209548` offset 45: the game's counter goes `0.08337 -> 0` and its end-of-frame speed
is 0, while the simulator kept the `0.08337` remainder, which flipped a pixel 35 frames later. **202 8
improved / 1460 identical / 0 regressed**, `+2,436` frames; **100pct 4 / 914 / 0**, `+1,239`; `1a`
unchanged.

The step before it, in the same round, was the **Summit intro flag** (`1fa7e8a`), which is where the
four inert rounds before it finally paid off: `3b238de` added the Summit landing rest and `0264120`
threaded `StateMachine.PreviousState`, and neither could take effect while two phase writes dropped
`INTRO_PHASE_SUMMIT_FLAG` on the way into the fall.

This is also the round that shows why the two commits before it were inert: `3b238de` added exactly
that 0.35 s rest and `0264120` threaded `StateMachine.PreviousState`, and **neither could take effect
while the phase chain never reached the Summit landing path**. Four rounds were spent inferring the
cause from the metric; an env-gated print of `intro_phase` at the divergence found it in one run. When
a phase machine is involved, print the phase.

The step before that was **`AscendManager`** (`753370f`), the Summit's ascent takeover.
`SummitBackgroundManager` had been filed in the entity registry's decoration bucket ("visual/audio, no
gameplay collider"), but it is `Celeste.AscendManager`, whose `Routine` (`AscendManager.cs:249-273`)
waits while `player.Y > base.Y` and then takes the player over as a dummy: `Speed = Vector2.Zero`,
`StateMachine.State = 11` (`StDummy`), `DummyGravity = false`. Six `StSummitLaunch` segments therefore
kept climbing at -240 px/s after the game had stopped - `pos+speed+state | anchor=StSummitLaunch` was
6 segments / 3,647 frames and is now **empty**. Measured `202 6 improved / 1462 identical / 0
regressed`, `+664` frames; `100pct 3 / 915 / 0`, `+332`; `1a` unchanged. `7-Summit|0|b-09` replays 848
frames instead of 730 and now stops on the ordinary one-pixel class.

The step before that was the **hurtbox for `PlayerCollider` entities** (`4090043`), and it is the largest
single win so far. `Player.Update` polls every `PlayerCollider` inside a block that swaps the player's
collider to the live hurtbox (`Player.cs:1898-1909`), so no callback ever sees the taller hitbox.
`interact` already did that for spikes, springs, boosters, spinners, killboxes, feathers, lava and the
core switch - but four `PlayerCollider` entities were still tested against the hitbox: `Refill`
(`Refill.cs:54`), `HeartGem`, `Puffer` and `Strawberry`. The hurtbox is `Hitbox(8f, 9f, -4f, -11f)`
against the hitbox's `(8f, 11f, -4f, -11f)`, so the whole difference is two pixels at the player's
feet - and it is enough to collect a crystal a frame early, which is what round 8's tracer had
localised but mis-attributed to the game's `RefillRoutine` coroutine. Measured **202 33 improved /
1435 identical / 0 regressed**, `450 -> 470` `ok`, `+4,832` frames; **100pct 21 / 897 / 0**,
`298 -> 311`, `+2,898`; `1a` unchanged. The dash-count class fell from 31 segments to 16 and dropped
out of the report's top six classes. Three unit tests had encoded the old behaviour - they placed the
player where only the hitbox touched the crystal - and now stand inside it, which is what the game
requires.

The step before that was the **dash-capacity witness** (`c0d8167`), and it is the second-largest single
win so far. `observe_session_dashes` combined the trace's witness of the session's
`PlayerInventory.Dashes` with the per-area table using `min`, which throws the witness away exactly
when the chapter has *raised* its capacity mid-play: `PlayerInventory.Farewell` is 1 dash, and
Farewell's own intro writes `Session.Inventory.Dashes = 2` directly
(`CS10_Gravestone.cs:133-134`, `:144-145`) before `CS10_FinalRoom.cs:76` returns it to 1. So
`min(2, 1)` restored a capacity of 1 and the simulator could never reach the 2 the game was using.
The witness is sound as a floor (`RefillDash` writes `Dashes = MaxDashes`, `BadelineBoost` increments
only up to `Inventory.Dashes`, and the pink `twoDash` diamond - the one refill that ignores
`MaxDashes` - is already excluded by position), so the two are now combined with `max`: **202 37
improved / 1431 identical / 0 regressed**, `437 -> 450` `ok`, `+2,800` frames; **100pct 37 / 881 / 0**,
`285 -> 298`, `+2,800`; `1a` unchanged. The `sim=1 game=2` half of the dash-count class fell from 41
segments to 8 and the class from 49/4,056 to 31/3,294.

The step before it in the same round was **un-flooring `Stamina`** (`c0b9cda`): the simulator clamped
both climb drains with `.max(0.0)` while `Player.cs:2648`, `4060` and `4078` are bare subtractions.
That clamp is invisible to every `Stamina <= 0` test downstream, so it changed no behaviour - it only
made the recorded value disagree with the game on each frame of a climb that ran past zero. It was the
**entire** `stamina | anchor=StNormal` class (16 segments whose only mismatch was the value: sim `0`,
game `-0.33` to `-10.83`): `202 15 / 1453 / 0`, `+1,600` frames; `100pct 7 / 911 / 0`, `+726`. Worth
re-reading every clamp in the simulator this way: a floor no branch can observe still costs frames,
because the gate compares the *value*.

The step before that was **`CoreModeToggle` and `Level.CoreMode`** (`8c201f9`). The Core's
ice/fire state was being read from the wrong field: `WallBooster.IceMode`
(`WallBooster.cs:77-101`), the ice factor (`Player.cs:3681-3684`) and every `CoreModeListener` read
`Level.CoreMode`, while the trace exported - and the simulator restored - `Session.CoreMode`. They are
separate fields and disagree on **3,641 Level rows of the 100% trace**, because `CoreModeToggle`
(`CoreModeToggle.cs:6-137`) flips `Level.CoreMode` mid-room and the entity decoded to `Unknown`.
The exporter now writes `levelCoreMode` beside `coreMode`; the gate prefers it and has
`--session-core-mode` to read the old key, which is how the change was measured against the *same*
trace. Per-segment: **202 7 improved / 1,461 identical / 0 regressed (421 -> 426 `ok`), 100pct 4 / 914
/ 0 (277 -> 280), 1a unchanged**, `+580` and `+318` replayed frames. That also retires the round-2
caveat that the conveyor's ice-mode `ClimbBlocker` cost 526 frames: with the right field plus the
toggle it is exactly neutral, and it is now modelled.

The step before that was the **input-press model** (`45b3070`, plus the
effective-press commit): the harness was feeding the game's already-buffered
`VirtualButton.Pressed` level into `Simulator`'s own `VirtualButton` buffer, so
every press outlived the game's by four frames. Per-segment diff against the
previous baselines: **1a 1 improved / 19 identical, 100pct 26 improved / 892
identical, 202 36 improved / 1432 identical, zero regressions on all three**,
for `+3 / +2,599 / +3,945` replayed frames and `+0 / +10 / +16` `ok` rooms. See
"The harness double-counts Monocle's `VirtualButton` buffer" below for the
ground-truth proof; that paragraph's original conclusion (that the fix had to be
two-sided) was **superseded** - adopting the recorded `Pressed` level verbatim is
enough.

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
4. **164 vanilla entity names still decode to `EntityKind::Unknown`** — no solid, no diagnostic.
   `tools/tas-fidelity/undecoded-entities.mjs` (output checked in beside it as
   `undecoded-entities.md`) scans the 27 vendored maps for every `Solid` subclass and `Trigger`
   subclass name the decoder does not handle, so the gap is a ranked list rather than a number. The
   ones with the strongest map evidence are `swapBlock` (8 maps), `switchGate` (18), `triggerSpikes`
   (6), `dashSwitch` (5), `lockBlock` (5), `fireBarrier` (9-Core/9H-Core) and `floatySpaceBlock` /
   `crumbleWallOnRumble` (LostLevels). Note the trap that cost two rounds: `SummitBackgroundManager`
   was in the *decoration* bucket while being a real `AscendManager`, and `Refill`/`HeartGem`/
   `Puffer`/`Strawberry` were polled with the wrong collider. Check the bucket and the collider before
   assuming a mechanism is missing. `map.rs` also does not decode the room `space` attribute.
5. **The real game is not bit-reproducible.** Two runs of the *same* exporter differ in ~28k rows,
   always starting at `7-Summit|a-00-intro`'s `StDummy` dummy walk (whole-pixel offsets with
   bit-identical `Speed`/`movementCounter`). "Frame-for-frame identical to vanilla" therefore has a
   noise floor that this environment cannot go below.
6. **The harness's input-press model fed the game's buffered `*P0` level into `Simulator`'s own
   `VirtualButton` buffer**, so every press lived four frames too long. **Landed** via
   `InputState::presses_are_effective`, which makes the simulator adopt the recorded `Pressed` level
   verbatim instead of re-buffering it: `1a +3`, `100pct +2,599`, `202 +3,945` replayed frames and
   `+0 / +10 / +16` `ok` rooms, zero per-segment regressions. Residual: the simulator still has no
   `VirtualButton.consumed` flag, so a press it consumes inside a frame stays visible to later reads
   of the *same* frame (`VirtualButton.cs:153-157`). That is now a small, bounded gap rather than a
   four-frame offset.

**How to continue.** Read the sections below for the environment, the ground rules for reading the
artifacts (never whole-file `JSON.parse` a trace), and the reproduce commands. New mechanics should
follow the established loop: pick a cluster from the gate report, cite the `Player.cs` line, prove
zero per-segment regressions, commit in a worktree, integrate.

**Three systematic levers have landed, each found from one divergence rather than from fitting the
report: the input-press model, the Core conveyor, and the Core's ice/fire state.** The first two are
described below (`+3 / +2,599 / +3,945` and `+0 / +1,414 / +2,848` replayed frames on
`1a / 100pct / 202`). The third is the most instructive: the trace was exporting `Session.CoreMode`
while every Core mechanic reads `Level.CoreMode`, and the two disagree on 3,641 rows because
`CoreModeToggle` - a decoded-as-`Unknown` entity - flips the *level* value mid-room. Measuring it
needed no new game run at all: export both keys in one trace and add a flag that reads the old one,
so the same trace produces both reports and the comparison has zero run-to-run noise. Next: re-run
`tools/tas-fidelity/lib/worklist.mjs` on `gate-final-202.json` (the class sizes have moved again), and
note that the same trick - export the field the source actually reads, alongside the one already
exported - applies to `Level.InSpace`, which `map.rs` still does not decode.

**The dominant remaining mechanism is a 1-pixel rounding difference.** `tools/tas-fidelity/lib/worklist.mjs` groups every `mismatch` segment of a report into classes (run it as `node tools/tas-fidelity/lib/worklist.mjs <report.json> <out.md>`; it streams). On the `395 ok` master the two biggest classes are `pos|anchor=StNormal` (180 segments / 11,340 frames) and `pos|anchor=StDash` (131 / 8,777), and their position deltas are overwhelmingly **one pixel on one axis** - 142 of the 180 are `(0,+-1)` or `(+-1,0)`, and 82 of the 131 likewise. That is the signature of sub-pixel remainder drift that stays invisible while the gate ignores `movementCounter` and only surfaces when it flips a `Math.Round` step, so these two classes almost certainly share a single root cause in the pixel-move / collision boundary code rather than hundreds of independent bugs. `dashes|anchor=StNormal` (43 / 3,708) is different: every one of its deltas is `(0,0)`, i.e. the divergence is reachable only through the dash count, not through motion.

**Rounding-mode lead: TESTED AND ELIMINATED.** The hypothesis was that the simulator used Rust's
`f32::round()` (half-away-from-zero) where C#'s `Math.Round` / `Monocle.Calc.Round` uses banker's
rounding (half-to-even). That is **already correct in the code**: the pixel-split path is
`sim.rs:8190-8192` —

```rust
*remainder += amount;
let amount = remainder.round_ties_even() as i32;   // == C# Math.Round, half-to-even
*remainder -= amount as f32;
```

so `round_ties_even()` *is* C#'s `Math.Round` semantics, and the five bare `.round()` sites
(`936`, `962`, `6256`, `9546`, `9547`) are outside the pixel-move path (cassette beats, dash-frame
counting, transition-speed rounding). **Do not chase rounding mode for the dominant one-pixel class.**
That leaves the alternative reading: a **missing or extra `Move` call**, which the recovered per-frame
total names directly — `T_axis = Δpos_axis + movementCounter_after − movementCounter_before`, where a
surplus of exactly `amount * dt` identifies the missing/extra `Player.cs` `MoveH`/`MoveV` and an
integer surplus identifies an exact move that should or should not have happened. Note also that
`movement_remainder` is *not* the same quantity as `Actor.movementCounter`'s C# sign convention —
verify the two agree before attributing a surplus to a missing call.

**Trap: the per-segment divergence *position* carries no information.** On the `395 ok` report all
**1,072** mismatch segments have `exactPrefixFrames == frames - 1`. That is not a measured property
of the game: the gate stops each segment at its first divergence, so a mismatch segment always
reports exactly one frame more stepped than matched. It says nothing about where in the room the
divergence happened. The number that does is
`records - 1 - leadingSkippedFrames - frames` (rows of the room that were never attempted); for
`1-ForsakenCity|0|5|755` that is 50, i.e. a genuine mid-room divergence. Reading the `frames - 1`
pattern as "every segment fails on its last row" is wrong and nearly produced a whole theory.

**The harness double-counts Monocle's `VirtualButton` buffer (proven on ground truth, then fixed).** `Monocle/VirtualButton` (`vendor/celeste-fna/Monocle/VirtualButton.cs:43-65`, `:107-146`) is
a *buffered* button: `Update` re-arms `bufferCounter = BufferTime` **only** when the raw
`Binding.Pressed` edge fires, decays it by `Engine.DeltaTime` every frame, and zeroes it whenever the
binding is not held; `Pressed` is true for as long as `bufferCounter > 0`. The trace's `*P0` keys are
exactly that buffered level, sampled before anything in the frame can consume it. `Simulator::step`
keeps its own equivalents (`jump_buffer_timer`, `dash_buffer_timer`, `crouch_dash_buffer_timer`) and
re-arms them from `input.*_pressed` on *every* frame that flag is true, so feeding `*P0` as
`input.*_pressed` (what `InputRec::to_input_state` does) makes the simulator's effective press the
union of the game's press window with the four frames that follow it.

Ground truth, `1-ForsakenCity|0|5`, trace rows 876-886 of `trace-1a-v4.jsonl`: `in.jump` (held) is
true from row 878, `in.jumpP0` is true on rows **878-882 only**, and the player is crouched
(`collider` height 6) and rising. The game swallows that press on all five frames - at row 883 a
`WallJumpCheck(-1)` wall finally comes into range, but by then `Input.Jump.Pressed` is false, so the
game keeps `speed = (333.666, -24.999)` and `Ducking = true`. The simulator, whose buffer was
re-armed through row 882, still holds a live press at 883, fires `Player.NormalUpdate`'s `WallJump`,
and reports `speed = (130, -105)` with `ducking = false` - the segment's entire divergence, and an
input-model artifact rather than a `Move` or rounding difference.

Measured alternatives on `trace-1a-v4` (same binary, same map directory):

| `input.*_pressed` handed to `Simulator::step` | `ok` | replayed frames | frame-exact |
| --- | ---: | ---: | ---: |
| level of `*P0` (what master does) | **16** | **2,126** | **2,122** |
| rising edge of `*P0` | 8 | 1,678 | 1,666 |
| rising edge of the held flag | 4 | 1,334 | 1,318 |

Both exact-edge models are *further* from the game than the level, and that decided the fix.
The simulator has no `consumed` flag (`VirtualButton.cs:153-157`) and no per-read-site re-read, so a
press it consumes (`wall_jump`/`jump`/`begin_dash` zero the timer) is gone for the whole frame, and
`ConsumeBuffer`'s zeroing (`Player.cs:2551`) has no equivalent at all. Reconstructing raw edges
therefore throws away information the simulator needs. The fix went the other way instead:
`InputState::presses_are_effective` (`types.rs`) tells `step` that these `*_pressed` flags **are**
Monocle's `Pressed` level - already consumed where the game consumed it - so it adopts them verbatim
instead of running them through a second buffer. `jump_held`/`grab_held` stay the raw
`VirtualButton.Check`, which `Player.cs:2952` and `2963` read for half-gravity and variable-jump.
Portable scenario inputs and the FFI pod leave the flag false and keep the buffered-edge model, so no
existing caller or test changes behaviour. Measured per-segment: **1a 1 improved / 19 identical /
0 regressed, 100pct 26 / 892 / 0, 202 36 / 1432 / 0** (`+3`, `+2,599`, `+3,945` replayed frames and
`+0`, `+10`, `+16` `ok` rooms), with `7-Summit|0|b-09` alone going from 5 to 730 frames.

An earlier revision of this paragraph concluded the fix had to be two-sided. That was wrong, and the
reason is worth keeping: the over-long press window was not compensating for a *missing* mechanic,
it was compensating for the simulator reading a press the game had already consumed - which adopting
the recorded level fixes directly.

**The dash-count class has two opposite sub-causes (measured).** Of the segments whose only divergence reason is dashes, 66 in total on the 395-ok report, the (simulator, game) dash pairs are: sim=1 game=2 in 35 segments, sim=1 game=0 in 21, sim=2 game=1 in 3, sim=2 game=0 in 3, sim=0 game=1 in 4. By area: LostLevels 34, 6-Reflection 12, 7-Summit 6, 1-ForsakenCity 6, 3-CelestialResort 3, 9-Core 3, 4-GoldenRidge 1, 5-MirrorTemple 1. So the class is not one bug: the 35 sim-lower cases look like a two-dash session whose refill the simulator never reaches (MaxDashes = Inventory.Dashes is 2 in areas 6/7/9 and for LostLevels after CS10_Gravestone, 1 for LostLevels before it), and the 21 sim-higher cases look like a refill the simulator performs where the game does not. Both live in the same refill sites (Player.RefillDash, UseRefill, BadelineBoost.cs:145-152), so a single audit of every refill site against its Player.cs MaxDashes/NoRefills guard should retire most of the class. This is independent of the one-pixel class.

**Three more classes are each confined to a single chapter (measured).** (1) pos|anchor=StDreamDash, 26 segments / 2,633 frames: 23 of the 26 are in 2-OldSite and the vertical position deltas are 3-4 px, so this is the Mirror Temple dream-dash exit or timer rather than rounding. (2) speed|anchor=StClimb, 16 segments / 2,183 frames: all 16 are in 9-Core and every one has a (0,0) position delta, i.e. the divergence is climb speed only, on the ice chapter - almost certainly one mechanism tied to the Core mode or the ice factor, and the coreMode restore is now in place so it is measurable. (3) dead|anchor=StDummy, only 2 segments but 1,710 frames, both in 5-MirrorTemple with (0,0) position deltas: a death or dummy interaction in the mirror rooms that costs a thousand frames per segment, so it is worth far more than its segment count suggests. Because each of these sits in one chapter, they are good parallel workstreams: one chapter each, no cross-talk with the pixel class or the dash-count class.

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
| `trace-202-v4` | after clutter / `CrushBlock` / the 184->180 clamp | 1,468 | **395** | 1,072 | 0 | **127,369** | **126,267** |
| `trace-100pct-v4` | later waves landed (same trace) | 918 | **262** | 656 | 0 | **76,600** | **75,928** |
| `trace-1a-v4` | same build | 20 | **16** | 4 | 0 | **2,126** | **2,122** |
| `trace-202-v4` | after the effective-press input model | 1,468 | **411** | 1,056 | 0 | **131,314** | **130,227** |
| `trace-100pct-v4` | same build | 918 | **272** | 646 | 0 | **79,199** | **78,536** |
| `trace-1a-v4` | same build | 20 | **16** | 4 | 0 | **2,129** | **2,125** |
| `trace-1a-v4` | same build | 20 | **16** | 4 | 0 | **2,129** | **2,125** |
| `trace-202-v5` | after `CoreModeToggle` + `Level.CoreMode` | 1,468 | **426** | 1,041 | 0 | **134,732** | **133,660** |
| `trace-100pct-v5` | same build | 918 | **280** | 638 | 0 | **80,926** | **80,271** |
| `trace-1a-v5` | same build | 20 | **16** | 4 | 0 | **2,129** | **2,125** |
| `trace-202-v5` | after `Level.InSpace` | 1,468 | **426** | 1,041 | 0 | **134,788** | **133,716** |
| `trace-100pct-v5` | same build | 918 | **280** | 638 | 0 | **80,957** | **80,302** |
| `trace-1a-v5` | same build | 20 | **16** | 4 | 0 | **2,129** | **2,125** |
| `trace-202-v5` | after `AscendManager` | 1,468 | **470** | 997 | 0 | **144,684** | **143,656** |
| `trace-100pct-v5` | same build | 918 | **311** | 607 | 0 | **87,713** | **87,089** |
| `trace-1a-v5` | same build | 20 | **16** | 4 | 0 | **2,129** | **2,125** |

The `v5` traces are `v4` plus one exported key, `levelCoreMode` (`Level.CoreMode`); replaying them
with `--session-core-mode` reproduces the `v4` numbers exactly (`918/918` and `1468/1468` segments
identical), so the trace change alone moves nothing and the two revisions stay comparable.

The `100%` row moved from 234 to **262** as the later waves landed, so always compare against a
named report file, not against the number in an older revision of this table. The three current
baselines are `r14b-202-v4.json`, `z1-base-100pct-v4.json` and `z1-base-1a-v4.json`, all under
`D:\celeste-research\.tmp\tasrun\`; `.tmp\tasrun\classes\q-segdiff.mjs <baseline.json> <candidate.json>`
prints an exact per-segment diff keyed by `(sid, mode, room, startRow)`.

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
* **`AscendManager` (map name `SummitBackgroundManager`) is modelled** (this round). It had been filed
  as a decoration; it is the Summit's ascent takeover (`AscendManager.cs:249-273`), and modelling it
  emptied the `pos+speed+state | anchor=StSummitLaunch` class (6 segments / 3,647 frames). `202
  6/1462/0`, `100pct 3/915/0`, `+664` and `+332` frames, `1a` unchanged. `index == 9`'s 1.6 s delay is
  not modelled; no corpus room uses it.
* **`PlayerCollider` entities are polled with the hurtbox** (previous round). Four of them - `Refill`,
  `HeartGem`, `Puffer`, `Strawberry` - were still tested against the two-pixels-taller hitbox, which
  collects a crystal a frame early. `202 33/1435/0`, `100pct 21/897/0`, `+4,832` and `+2,898` frames.
  The lesson generalises: when a divergence is one frame and one pixel, check *which collider* the
  source uses before hunting for a missing call. Three unit tests had baked in the wrong collider.
* **The dash-capacity witness now raises the area floor** (previous round). `min` -> `max` in
  `observe_session_dashes`: a chapter that raises `Session.Inventory.Dashes` mid-play (Farewell's
  intro, `CS10_Gravestone.cs:133-134`) was being clamped back to the area's 1. `202 37/1431/0`,
  `100pct 37/881/0`, `+2,800` frames on both, `1a` unchanged. The dash-count class fell from
  `49 segments / 4,056 frames` to `31 / 3,294`, its `sim=1 game=2` half from 41 to 8.
* **`Stamina` is not floored** (this round). `Player.cs:2648`, `4060`, `4078` subtract from a bare
  field, so a climb that outlasts the bar leaves a negative value; the simulator's `.max(0.0)` was
  invisible to every `Stamina <= 0` branch and still cost `+1,600` replayed frames because the gate
  compares the value. `202 15/1453/0`, `100pct 7/911/0`, `1a` unchanged. Guarded by
  `climb_drain_lets_stamina_go_negative_like_the_source`.
* **`Level.InSpace`** (previous round). Restored from the anchor row's `inSpace` (the room sets it once at
  load, so it is constant per segment) and applied at all four physics sites - run target, both fall
  caps, `NormalUpdate` gravity and `DummyUpdate` gravity. `202 4/1464/0, 100pct 2/916/0, 1a 0/20/0`,
  `+56` and `+31` replayed frames, all of it in the one vanilla `space` room. Guarded by
  `space_rooms_scale_run_target_fall_caps_and_gravity`.
* **`CoreModeToggle` and the `Level.CoreMode` ground truth** (previous round). The switch decoded to
  `Unknown`, so `Level.CoreMode` never changed inside a room and the Core's ice/fire state was frozen
  at the value the segment anchored on - while the trace exported the *session* value, which is a
  different field. Now: `levelCoreMode` is exported and preferred, `CoreModeToggle` is decoded
  (`Hitbox(16, 24, -8, -12)`, `onlyFire`/`onlyIce` in `direction`, `persistent` in `single_use`) and
  modelled (`Usable && cooldownTimer <= 0` flips the mode, `Celeste.Freeze(0.05f)`, one-second
  cooldown counted down after `Player.Update`). `202 7/1461/0, 100pct 4/914/0, 1a 0/20/0`
  (improved/identical/regressed), `+580` and `+318` replayed frames. Guarded by
  `core_mode_toggle_flips_level_mode_then_cools_down`.
* **The Core `WallBooster` conveyor** (previous round). Entity decode plus
  `Player.ClimbUpdate`'s booster branch: `Speed.Y` toward `WallBoosterSpeed`
  (-160) at `WallBoosterAccel` (600), `LiftSpeed = UnitY * Max(Speed.Y, -80)`
  (`Player.cs:3093-3099`, `3154-3167`), the `wallBoosting` release on the ledge
  branch (`Player.cs:3140-3149`), and `ClimbBlocker.EdgeCheck` narrowed to
  `InvisibleBarrier` because the booster's blocker is `edge: false`
  (`ClimbBlocker.cs:40-50`, `WallBooster.cs:42`). `202 18/1450/0, 100pct 9/909/0,
  1a 0/20/0` (improved/identical/regressed), `+2,848` and `+1,414` frame-exact
  frames. Guarded by `wall_booster_ramps_climb_speed_and_publishes_lift_speed`.
* **The input-press model** (this round). `InputState::presses_are_effective` (`types.rs`): the TAS
  fidelity harness exports `Monocle.VirtualButton.Pressed` itself (`*P0`, already zeroed wherever the
  game called `ConsumeBuffer`/`ConsumePress`), so `Simulator::step` must adopt that level verbatim
  instead of re-arming its own buffer from it. `1a 1/19/0, 100pct 26/892/0, 202 36/1432/0`
  (improved/identical/regressed) and `+3, +2,599, +3,945` replayed frames. `7-Summit|0|b-09` went
  from 5 to 730 frames; 10 more `ok` rooms on `100pct`, 16 more on `202`. Portable inputs and the FFI
  pod leave the flag false, so no existing caller changes behaviour.
* **The `CanUnDuck` gate on `NormalUpdate`'s wall-jump branch** (this round). `Player.cs:2969-3004`
  puts the wall-jump / `ClimbJump` / water-jump branch inside `else if (CanUnDuck)`, so a crouched
  player whose normal hitbox does not fit where it stands swallows the press entirely and keeps
  `Ducking = true`; the simulator ran the branch unconditionally and turned the same press into a
  `WallJump` (speed `WallJumpHSpeed`/`JumpSpeed`, `Ducking = false`). Guarded by
  `crouched_jump_press_is_swallowed_while_can_unduck_is_false` in `sim.rs`, which fails if the gate is
  removed. **Measured effect on the corpus: none** - a per-segment diff over all three traces is
  byte-identical (`1a` 20/20, `100pct` 918/918, `202` 1468/1468 segments unchanged, same `frames`/
  `exactPrefixFrames`/`stalledFrames`), because the press the game swallows is the same press the
  harness currently mis-arms anyway (see the `VirtualButton` finding above). It is landed as a
  faithful transcription plus a real unit test, and it is a prerequisite for evaluating the
  input-model workstream.

### Known open gaps (measured, not guessed)

* **The one-pixel `pos` classes now have a minimal two-frame repro, and it is a collision asymmetry
  rather than rounding.** `4-GoldenRidge|0|c-06b|52462` (A-side, room bounds x 7296..7616
  y -3256..-3076) replays one frame exactly and stops on the second:

  | frame | game | simulator |
  | --- | --- | --- |
  | 52503 (offset 0) | pos (7604,-3102), counter (0, 0.25), move (-0.66665, -1.75) | identical |
  | 52504 (offset 1) | pos (7603,-3104), counter (-0.16667, 0.49999), move (**-1.16667**, -1.75) | pos (7604,-3104), counter (0, 0.49999), move (**0**, -1.75) |

  Both sides jump on the same frame (`Speed.Y = -105`, `varJumpSpeed`/`varJumpTimer` agree) and the
  y amount, counters and final y are bit-identical; only x differs, and the simulator's x amount is
  exactly `0`, i.e. its `MoveH` was blocked at the first pixel. A `DSH_MOVE_TRACE` instrumentation of
  `move_axis_amount` (since reverted) named the blocker exactly: `h=true pos=(7604,-3102)
  next=Rect { x: 7599, y: -3113, w: 8, h: 11 } tile=true dream=false jt=false ents=[]` - a **tile**,
  not an entity.

  The room's raw `solids` innerText (read out of the `.bin`, one character per 8 px cell, rows
  trimmed of trailing empties) has `f` at that cell: the room's origin is (7296,-3256), so cell
  (37,19) is `Rect { 7592, -3104, 8, 8 }` and row 19 reads `...00f` at columns 35-37. So the tile is
  in the map text and the simulator's decode of it is right; what differs is that the game's player
  moved into it.

  The frame-order candidate is now **excluded on the simulator's side**: `move_axis`
  (`move_axis_amount(p, map, horizontal, speed * dt)`) is called with `horizontal = true` and then
  `false`, which is the source's `MoveH`-before-`MoveV` order (re-checked in the vendored
  `Player.cs`), so the simulator is not transposing the axes. What the frame *is* can be read off its
  speeds: `Speed.X = -70` from a pre-jump `-30` plus `moveX * JumpHBoost` (`-40`) together with
  `Speed.Y = JumpSpeed` (`-105`) is exactly a `ClimbJump`/wall-jump launch off that wall, after which
  the game moved one pixel left into the tile's column.

  So the open question is specific and **decidable from the trace with no new code**: is the game's
  cell (37,19) solid at all? Walking the room visit does not settle it, and two earlier guesses are
  now withdrawn rather than left standing:

  * The game's frame applies a **full `-1.16667` horizontal move**: its `movementCounter.X` ends at
    `-0.16667`, which a blocked `MoveH` cannot produce (`Actor.MoveHExact` zeroes the axis counter, and
    `Player.OnCollideH`'s ordinary branch never re-applies the amount; only state 19 and a
    `DashAttacking` dash-collision branch alter it, and `Speed.Y = -105` with `Speed.X = -70` is a
    `ClimbJump`). So the game's probe at `[7599, 7607] x [-3113, -3102]` - which overlaps
    `[7592, 7600] x [-3104, -3096]` - found nothing.
  * That cannot be reconciled with the map by blaming placement: `tile_rects` splits on lines and reads
    each row with `chars().nth(x)`, so ragged rows (trailing empty cells trimmed) are handled, and the
    cell is `f` while `f` is the same character as the floor row the player stands on one row below.
  * Nor is the jump a witness: the source's climb-jump branch in `NormalUpdate` requires only `Facing`,
    `Stamina > 0`, `Holding == null` and *no blocker eight pixels in front of the facing direction*; it
    never re-probes a wall on the facing side.

  What is left is that some other quantity on that frame differs inside the simulator - its fed
  `move_x`, or the pre-jump `Speed.X` (`-30` in the game) - and the round-13 tracer could not tell,
  because it printed no frame index and its three lines cannot be attributed to the diverging frame
  with certainty.

  `--dump-segment` now prints both sides' speeds, and that closes the question it was asked to close:
  at row 52504 the game holds `gameSpeed=(0.00000,-105.00000)` and the simulator holds
  `rustSpeed=(0.00000,-105.00000)` - **identical, including `Speed.X = 0`** - while the game's x
  displacement is `-1` with its counter ending at `-0.16667` and the simulator's is `0` with its
  counter at `0`. So the simulator is not missing the speed, and its own `Speed.X = 0` at the end is
  what a blocked `MoveH` plus `OnCollideH` produces; the game ends at `Speed.X = 0` *without* a
  counter zeroing, i.e. through a path that displaces x by `-1.16667` and then clears the speed
  anyway. The recovered move decomposes either as one successful `MoveH(-1.16667)` or as a raw `-1`
  position write plus a `-10 * dt` move; the next probe is that decomposition - read
  `Player.OnCollideH`'s normal branch and `Actor.MoveHExact` for a path that both moves and clears
  `Speed.X` - and not the tile.

  Found along the way and still open: `add_room_edge_tile_bleed` copies the room's boundary tiles
  outward without the source's occupancy guard (`LevelLoader.cs:233-264` stops each propagation at
  the first non-empty target cell). The guard cannot be reproduced from the current room alone - the
  source runs that loop over *every* room in the map against the shared grid, so a faithful version
  has to decode all rooms' `solids` layers, which the simulator does not - and the missing direction
  it implies (a *neighbour's* edge tiles bleeding into this room) is a second, separate gap.

* **The state-reason classes have a by-design half, and it is now separable.** `speed+state |
  anchor=StNormal` is 41 segments / 2,762 frames, but `tools/tas-fidelity/class-states.mjs` (new) shows
  28 of those segments and 1,876 of those frames are the Badeline boss's `StAttract` in 6-Reflection -
  an intentional product exclusion, since `Attract` is one of the two excluded mechanisms in
  `AGENTS.md` - and the fixable remainder is `StLaunch` (8 / 688, all 9-Core C-side), `StRedDash`
  (2 / 85), `StTempleFall` (1 / 73) and `StFlingBird` (2 / 40). `state | anchor=StNormal` is likewise
  1,454 frames of which `StIntroJump` is 5 segments / 861 frames (7-Summit x4, 6-Reflection x1) and
  `StIntroWakeUp` 2 / 376, against 183 frames of `StDummy` and 34 of `StIntroRespawn`. Always run a
  state class through that tool before treating its frame count as a work estimate.
* **`StLaunch` is not a missing mechanic, so do not "add" it.** The only three writers of state 7 are
  `Player.ExplodeLaunch` (`Player.cs:4967`), `FinalBossPushLaunch` (`:4982`, excluded) and
  `BadelineBoostLaunch` (`:4999`), and the launcher in the 9-Core C-side room `01` is a `Bumper`
  (`Bumper.cs:170` calls `player.ExplodeLaunch(Position, snapUp: false)`), which `interact`'s
  `EntityKind::Bumper` arm already calls as `explode_launch(p, input, target, false, false)`. The 688
  frames are therefore a condition or ordering difference, not an absent state. **The specific
  ordering claim made here last round was wrong and is withdrawn**: `Player.Update` runs
  `base.Update()` (the state callback) at `Player.cs:1778`, the physics `MoveH`/`MoveV` at
  `:1799-1805`, and only then the `PlayerCollider` pass at `:1898-1914` - which is the same position
  the simulator's `interact` occupies relative to its physics. The pass itself also matches: it
  iterates *every* `PlayerCollider` (no early exit on a hit; it only returns when the player dies) and
  runs with the hurtbox installed.
* **`Level.EnforceBounds` is not modelled.** `Player.Update` calls it at `Player.cs:1915-1918`,
  immediately after the `PlayerCollider` pass, gated on
  `InControl && !Dead && StateMachine.State != 9 && EnforceLevelBounds`. `Level.EnforceBounds`
  (`Level.cs:2725-2790`) is not a simple clamp: it clamps `player.Left/Right/Top/Bottom` to
  `Bounds`, calls `OnBoundsH`/`OnBoundsV` when it does, and - before clamping - calls
  `Session.MapData.CanTransitionTo` and `NextLevel(...)` to hand off to the neighbouring room
  (four directions, each with a `player.Center +/- 8`/`12` probe and a `Before*Transition` hook),
  plus a `CameraLockModes.FinalBoss` variant (excluded) and a `TheoCrystal`-in-hand special case
  that clamps `Right` to `bounds.Right - 1`. The simulator has its own bound clamping and its own
  transition start, so this is a structural difference rather than a missing one-liner: whoever takes
  it on should diff the two structures rather than bolt on a clamp.
* **The intro states split into two problems, one of which is now fixed.** The `state |
  anchor=StNormal` class was 1,454 frames; `1fa7e8a` fixed the Summit half.

  * `7-Summit|1|g-00|209548` diverges on the **landing frame** of the Summit hand-off (offset 45 of
    46): the game's `movementCounter.Y` is zeroed by the landing collision and its state is still
    `StIntroJump`, while the simulator had already returned to `StNormal`. That is the missing
    `if (wasSummitJump) { ...; yield return 0.35f; }` rest (`Player.cs:6055-6067`), which `3b238de`
    adds - but the phase is unreachable today because the simulator *guesses* `wasSummitJump` from
    the entry speed and position in `intro_resume` rather than reading
    `StateMachine.PreviousState == 10` (`Player.cs:5998`), and `PlayerSnapshot` has no previous
    state. **Plumbing the previous state (the gate can take it from the row before the anchor) is the
    next step**, and it is what makes the rest observable.
  * `6-Reflection|0|after-01|103119` (677 frames, the largest single segment in the class) diverges
    on the **hand-off frame itself**: the game goes `StSummitLaunch` (-240) -> `StIntroJump` (-105)
    while the simulator goes `StNormal`, i.e. it never *chooses* an intro state. The game picks it
    when the new room loads, from `AreaData.IntroType` (`AreaData.cs`) via
    `Level.LoadLevel(Player.IntroTypes)` (overridable per cutscene). The simulator only ever
    *restores* the state at a segment anchor, so a room load that happens mid-segment is missed.
    Modelling that needs the chapter's `IntroType` table and the area available to the simulator.
    `previous_state` is now threaded too (`0264120`): the gate carries each segment's ending state
    to the next one, keyed by `(sid, mode, room)`. **The key matters** - the first version carried it
    unconditionally and regressed `1-ForsakenCity|0|1|273287` (`ok`/274 -> mismatch/75), because that
    late 1A `StIntroJump` segment followed `7-Summit`'s `StSummitLaunch` and the simulator took the
    Summit branch for a Prologue-style intro. With the same-chapter/different-room guard the corpus is
    clean again, but the threading is still **inert** (0 improved / 1468 identical / 0 regressed), so
    the Summit intro segments' missing piece is not `PreviousState` either. Stop inferring it from the
    metric: print `intro_phase` at the divergence for one of them (`--rooms g-00`) and read which
    phase the simulator actually chose.
* **The next divergence behind the Summit intro is a missing coyote-time extension, and it is
  identified down to the line.** `7-Summit|1|g-00|209548` now replays 328 frames (46 before the two
  fixes above) and stops at a `SuperJump`: the trace's inputs there are `jump=true, jumpP0=true,
  mx=+1` and the game's speed becomes `(260, -105)`, which is `Player.SuperJump`'s
  `SUPER_JUMP_H`/`JumpSpeed` pair. An env-gated print inside `dash_jump` shows the simulator refusing
  it with `jump=true unduck=true dashdir=(-1,0) grace=0.00000 wallR=false wallL=false` - so the only
  thing missing was the coyote time. `Player.cs:2379` is where the game extends it:
  `if (jumpGraceTimer > 0f) jumpGraceTimer = 0.6f;`, inside the `TransitionTo` interlude that also
  sets `AutoJump`, `varJumpSpeed = -60f` and `varJumpTimer = 0.15f` for a room hand-off that carries a
  jump intro. `update_transition` instead *zeroes* it (`p.jump_grace_timer = 0.0;`). Mirroring that
  hand-off is the next step, and it is worth checking the other `jumpGraceTimer` writers while there:
  `Player.cs:1584` (ground block, modelled), `:3007` (`StartJumpGraceTime`, whose only vanilla caller
  is `BounceBlock.cs:484`, modelled) and `:5169` (dream-dash exit, modelled).

  **The `:2379` attribution in this entry was wrong and is withdrawn**: that line is inside
  `Player.HiccupJump`, whose only caller is gated on `SaveData.Instance.Assists.Hiccups`
  (`Player.cs:1438`), an assist option the TAS does not use. The timer rule itself is `Player.Update`'s
  common block, `if (onGround) { dreamJump = false; jumpGraceTimer = 0.1f; } else if (jumpGraceTimer >
  0f) jumpGraceTimer -= Engine.DeltaTime;` (`Player.cs:1581-1589`), and the simulator already mirrors it
  in the common section of `step`.

  **The trace exports `jumpGraceTimer`**, which settles the whole question and replaces the guesswork
  above with data: on row 209880 the game has `onGround = true` and `jumpGraceTimer = 0` at the end of
  the frame - exactly what the ground block setting 0.1 and then `SuperJump()` consuming and zeroing it
  produces. So the coyote time was **zero on both sides**; what differs is the ground contact. The
  simulator's probe prints `pos=(12811.0,-11720.0) speed=(-275.17,0.00) state=Dash probed=false
  was=false grace=0.00000 duck=false` while the game's row has the player at **y = -11722** and
  `onGround = true`. Two pixels of floor contact, not a timer.

  **And that floor is a `crumbleBlock`.** `7H-Summit.bin` `g-00` holds
  `entity { name: "crumbleBlock", bounds: [12792, -11720, 16, 8] }` - the player's feet rest on its
  top - and the simulator decoded it as `Unknown`. `EntityKind::CrumbleBlock` now decodes it
  (`map.rs`), deliberately **inert**: wiring it into the solid lists as a plain solid measures **38
  improved / 6 regressed** on 202 (`484 -> 496` `ok`, `+2,977` frames, but `5-MirrorTemple|0|b-20`
  loses an `ok`), and the six are the segments where the TAS lingers on a block long enough for
  `CrumblePlatform.Sequence` (`CrumblePlatform.cs:94-169`) to collapse it: shake, `yield return 0.2f`
  per step (1 step on top, 3 while climbing), a further up-to-0.4 s while the player stays on top,
  then `Collidable = false`, `yield return 2f`, and re-arm once nothing overlaps. **That sequence is
  the work item**; the decode alone is groundwork.

  Also found by the same measurement and worth remembering: `Map::non_dream_solid_at` (`map.rs:2051`)
  keeps its **own** copy of the solid-kind list, separate from `sim::is_solid_entity`, and the ground
  probe only consults the former - so a kind added to `is_solid_entity` alone (as `StaticSolid` was in
  round 11, and `CrumbleBlock` was here) is **inert**. Keep the two lists in step, or derive one from
  the other.

  Implementation note for whoever takes the sequence, so it is not rediscovered: the state belongs on
  `Simulator` (it clones for `fork` automatically) rather than in `PlayerSnapshot`, which keeps the
  gate's `--dump-field-map` audit untouched; `Collidable = false` is the parked-bounds idiom
  (`park_entity`/`PARKED_ENTITY_POSITION`) that `DashBlock` already uses; and the free `step`
  function's signature has to carry the new state, because `advance_post_player_entities` is called
  from three separate branches inside it - wiring only the obvious call site would advance the
  coroutine up to three times in one frame.
* **The simulator has no `VirtualButton.consumed` flag.** With `presses_are_effective` the press
  level each frame is now exactly the game's, which retired the four-frame offset; what remains is
  that a press the simulator consumes *inside* a frame (`wall_jump`/`jump`/`begin_dash` zero the
  timer) stays visible to later reads of the same frame, where the game's `VirtualButton.Pressed`
  would report false (`VirtualButton.cs:153-157`). No read site depends on that yet, so this is
  recorded as a residual rather than a target.
* **The dash-count class is down to 16 segments** (from 49 two rounds ago): `sim=1 game=2` (8),
  `sim=1 game=0` (7) and `sim=2 game=1` (1), and it is no longer among the report's six largest
  classes. The `sim=1 game=0` half that round 8 chased was *mostly* the hurtbox bug above, not the
  `RefillRoutine` coroutine: the worked example `6-Reflection|1|b-04|188460` is now `ok`. What remains
  to check there, if it is chased again: the simulator still applies a collected crystal's `UseRefill`
  in the same frame, where `Refill.OnPlayer` (`Refill.cs:166-174`) starts `RefillRoutine`, which does
  `Celeste.Freeze(0.05f)` and `yield return null` before `player.UseRefill(twoDashes)`
  (`Refill.cs:178-190`); and `CanDash` (`Player.cs:1074-1088`) still lacks
  `(TalkComponent.PlayerOver == null || !Input.Talk.Pressed)`.
* **`pos+speed+state | anchor=StSummitLaunch` is closed** - `AscendManager` was its cause, and the class
  is empty. What the takeover exposes is the general shape of the remaining Summit work: after the
  dummy hand-over the cutscene drives the player itself (`DummyWalkTo`, camera moves), so any further
  Summit segment will need the cutscene's own script, not just the state machine.
* **The `space` room's remaining causes are `dashes` and `SpaceController`.** With `InSpace` landed,
  its four segments (`9-Core|0|space` x2, `9-Core|1|space` x2) replay 13-19 frames and then stop on a
  dash-count divergence, with `onGround` disagreeing on two of them - so the next causes there are the
  dash refill inside the shaft and `SpaceController` (`SpaceController.cs:14-29`), which wraps the
  player vertically against `Level.Camera` bounds and is not modelled. It is camera-dependent, and the
  camera model still has the residual recorded below, so it needs the camera checked first.
* **`Level.InSpace`** (`Level.cs:449`) is a per-room map property `map.rs` does not decode; the
  simulator reads it from the trace instead, so a portable scenario that does not come from a trace
  cannot set it.
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
| `trace-1a-v5.jsonl` (1A only, fast iteration) | 3,215 | 3,213 | — | 11.8 MB |
| `trace-100pct.jsonl` | 281,113 | 266,262 | 918 | 845 MB |
| `trace-202.jsonl` | 461,122 | 438,303 | 1,468 | 1.39 GB |
| `trace-100pct-v5.jsonl` | 281,113 | 266,262 | 918 | 996 MB |
| `trace-202-v5.jsonl` | 461,122 | 438,303 | 1,468 | 1.64 GB |

`v5` is `v4` plus one exported key, `levelCoreMode` (`Celeste.Level.CoreMode`), written by the same
`tools/celestetas-trace/apply.mjs` patch; `v4` traces stay usable because the gate falls back to the
older `coreMode` (`Session.CoreMode`) key when the new one is absent. Both v5 runs report
`sync-check status: success` and reproduce the v4 row counts exactly.
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
# then install that tree as a dev mod at <game>\Mods\CelesteTAS-EverestInterop\:
#   copy <tree>\CelesteTAS-EverestInterop\bin\Release\net8.0\* over
#   D:\celeste-research\.tmp\tasrun\game-trace\Mods\CelesteTAS-EverestInterop\bin\
#   (that folder's everest.yaml points DLL at bin/CelesteTAS-EverestInterop.dll)

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

## Undecoded solids, measured one kind at a time

Every entry is a single-kind change measured against the previous report on all three traces; a
batch mixes a real regression with inert additions and hides which is which.

| kind | result (202) | verdict |
| --- | --- | --- |
| `dashSwitch` | 0 improved / 0 regressed | kept, inert |
| `starJumpBlock` | 0 / 0 | kept, inert |
| `crumbleWallOnRumble` | 3 / 0 | kept |
| `seekerBarrier` | batch: 3 / 22 | reverted; a Solid for seekers, the player passes through it |
| `switchGate` | 11 / 2 | reverted; needs `SwitchGate.Open`, the +373 frames are its value |
| `floatySpaceBlock` | 7 / 1 (net +6) | reverted; one LostLevels segment loses a frame, so it needs its real motion |
| `resortRoofEnding` | 0 / 0 | kept, inert |

`theoCrystalPedestal` starts `Collidable = false` (`TheoCrystalPedestal.cs:21`), so it is not a
solid candidate at all.

