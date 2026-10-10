# TAS replay fidelity environment (vanilla 202 / 100%)

Objective: run the **vanilla Celeste full-game TAS** as a fidelity gate for `celeste-physics`
("next-gym"). The real game produces the ground truth; next-gym must reproduce it frame by frame.

## Status and handoff

**Where it stands.** The environment works end to end and the gate is real: the real game plays the
pinned vanilla 202-berry TAS, the instrumented CelesteTAS dumps one record per executed frame, and
`tas_fidelity` replays every room segment through `Simulator` and stops at its first divergence.

| trace | `ok` rooms | mismatch | unsupported | replayed frames | frame-exact |
| --- | ---: | ---: | ---: | ---: | ---: |
| `trace-202-v5` | **511** | 956 | 0 | **159,350** | **158,363** |
| `trace-100pct-v5` | **334** | 584 | 0 | **95,613** | **95,012** |
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
| `switchGate` | 12 / 2 | **now modelled** (`2902082`): the sequence plus `switches_<room>` threading; the 2 are the unrepresentable flag gap below |
| `floatySpaceBlock` | 7 / 1 (net +6) | reverted; one LostLevels segment loses a frame, so it needs its real motion |
| `resortRoofEnding` | 0 / 0 | kept, inert |
| `swapBlock` | 9 / 3 (net +184) | reverted; it moves, and the three losses are segments the player rides it (`5-MirrorTemple|0|a-01`, `b-10`, `7-Summit|0|f-10`) |
| `lockBlock` | 0 / 1 | reverted; the TAS unlocks it with a key and passes through, so it needs its unlock state |

The sweep is finished: every Solid subclass the audit found in the maps has now been measured one at a
time, and the ones left are all the same shape - a kind whose *state* is what matters
(switchGate's open, lockBlock's unlock, swapBlock's two-point motion,
loatySpaceBlock's motion, crumbleWallOnRumble's rumble). Adding any of them as a plain solid
measures a regression, so the next step on this line is a state machine per kind.

`switchGate` was the first and is now done (`2902082`): a solid running `SwitchGate.Sequence`
(`SwitchGate.cs:102-145`) - wait for every `Switch` in the room (`Switch.FinishedCheck`), `yield 0.1`, a
0.5 s icon ramp, `yield 0.1`, a 2 s `Ease.CubeOut` tween of `MoveTo(nodes[0])` quantized by *truncation*
the way `Solid.MoveTo` does, then `yield 1.8` - with `TouchSwitch` activating from its 30x30
`PlayerCollider` box (`TouchSwitch.cs:44-45`) and the room's `switches_<room>` session flag carried
across segments keyed by room.

**It is also the first deliberately kept regression, and the proof is the point.** 202 measured 12
improved / 2 regressed (the 100pct trace adds 7 / 1), and all three regressions are `2-OldSite|0|6`
losing **one frame**: that room's `touchSwitch` sits at x=336 while the replayed windows never take the
player there, so the game's gate is already open when the window starts (`SwitchGate.Awake`,
`SwitchGate.cs:68-82`) and the simulator cannot know it - the flag is `"switches_" + Session.Level`
(`Switch.cs`), the trace exports no session flags, and the triggering touch lies outside every window
the trace replays. A regressing change may be kept only with that kind of proof, in the commit message.

Do not re-run the sweep.

**A reverted change needs a rebuild before any fresh measurement.** The one-command sweep above
reverts the *source* when a kind regresses, but the example binary on disk keeps the reverted
behaviour until the next cargo build - so a fresh gate run can report the reverted kind's numbers
and look like a verification of the committed state. Round 39 hit exactly that (the fresh 202 run
printed swapBlock's 505 / 158,166); after rebuilding, the committed state reproduces
503 / 157,982 / 156,987 per segment exactly.

`theoCrystalPedestal` starts `Collidable = false` (`TheoCrystalPedestal.cs:21`), so it is not a
solid candidate at all.



## Round 45 recon results (four parallel subagents, all read-only)

**Wind ordering: implemented, measured net-negative, reverted.** The recon found that
`apply_wind_movement` (`sim.rs:5710`) runs before the `Player.Update` mirror while the source's
`Player.WindMover` is a component added *after* `StateMachine` (`Player.cs:1172` vs `:1180`), so every
`Player.WindMove` guard reads the previous frame's `State`/`Speed.Y`/`onGround`/`Ducking`. The evidence
was exact: gaps of `wind * 0.1 * dt` on 12 + 2 + 6 segments across the three biggest classes
(`pos|StNormal`, `pos|StDash`, `pos+speed|StNormal`). Moving the call to just after `tick_lift_speed`
measured **58 improved / 22 regressed, 158,407 -> 156,123 frames** and lost an `ok` in
`7-Summit|1|e-00`, concentrated in 7-Summit and 4-GoldenRidge - the windy chapters. **Reverted.** So the
stale-guard reading is real but something else in the simulator's frame order compensates; the next
attempt must be narrower than moving the application wholesale (e.g. evaluate the guards post-callback
while keeping the move where it is, or find the compensating site), and it must be measured per
chapter.

**`dashSwitch` is a dead map name - the arm never matches.** The vanilla names are **`dashSwitchH`**
and **`dashSwitchV`** (`Level.cs:608-610`); a byte scan of all 27 `.bin` files found the bare
`dashSwitch` in none. So `map.rs`'s `"dashSwitch" => EntityKind::StaticSolid` is unreachable and every
real dash switch arrives as `Unknown` - which also means the earlier "dashSwitch as a plain solid
measured 0 / 0" entry was a **no-op artifact**, not evidence. The real collider is `16x8` (Up/Down) or
`8x16` (Left/Right) chosen from `leftSide`/`ceiling` (`DashSwitch.cs:62-71`, `:109-121`), not the map
rectangle, so the fix is a decode arm per name. This is the cheapest high-value item left: the switch's
press is driven by the player's own dash collision (already modelled for `DashBlock`/`CrushBlock`) and
only its *persistent* session flag is unrepresentable.

**Session state worth exporting, ranked** (`Session.DoNotLoad` first - it is the only mechanism that
suppresses entity construction outright, `Level.cs:472`/`:1188`, and it is the input to
`conditionBlock condition:Key` and `ridgeGate`; then `Session.Cassette`, which makes the simulator
*provably* wrong because `CassetteBlock` starts non-collidable and the manager is never built once the
tape is taken; then `dashSwitch_<id>`, `oshiro_clutter_door_open`, `disable_lightning`). The exporter
now emits `Session.Flags` as a top-level `flags` array (append-only tail, sorted, backward compatible),
so the next game run gives the harness that input; nothing under `crates/` reads it yet.
## Round 46: the exporter rebuild path does not reproduce the working mod

Adding `Session.Flags` to `tools/celestetas-trace/TasFrameTrace.cs` and rebuilding from scratch
(`robocopy .tmp/tasrun/celestetas-src -> celestetas-trace` + `apply.mjs`) builds clean (0 errors, 6
warnings) and installs, and the 1A TAS still completes (`status: success`, `AreaComplete`, 35.9 s) -
but the trace file is **0 bytes**. The known-good exporter is the older `celestetas-trace-w8v5` tree,
whose `TasFrameTrace.cs` has `levelCoreMode`/`AppendInventory` but no `AppendSessionFlags`, and whose
DLL has been restored into `game-trace/Mods/CelesteTAS-EverestInterop/bin`.

So `apply.mjs` plus the current `celestetas-src` does **not** reproduce the working v5 exporter: the
`w8v5` tree carries something extra (a different upstream revision, or a manual patch `apply.mjs` does
not re-apply). The next attempt must patch **`celestetas-trace-w8v5`'s** `TasFrameTrace.cs` directly and
rebuild *that* tree, not re-derive it - and it must verify a non-zero trace before trusting a run.

Two checks to run first, both cheap: diff `w8v5/.../TasFrameTrace.cs` against the freshly patched tree
to see what is missing, and confirm whether the 0-byte trace is a dropped-row path (every row throwing
inside `WriteFrame`) or the command never running at all.

## Round 51: the v6 traces were exported with the WRONG (pre-flags) DLL

The 100% and 202 re-exports succeeded (1,044,809,413 B / 1:55 and 1,720,069,809 B / 3:35, both
`status: success`) but a scan of the first 120,001 rows of `trace-202-v6.jsonl` finds **no `flags` key**,
so those runs used the pre-flags exporter. Cause: round 46 restored the v5 DLL into
`game-trace/Mods/CelesteTAS-EverestInterop/bin`, and the loader refreshed `Mods/Cache` from it. The v6
DLL (built from `celestetas-trace-w8v5`, SHA256 DDAC7E16...33F4739, byte-identical to the DLL that
produced a working `flags` trace at 18:04) is now installed over it.

**Rule to keep: after exporting, assert the new key is present before believing the trace.** A build
timestamp or a `status: success` is not evidence; the flag array is.

Both `trace-100pct-v6.jsonl` and `trace-202-v6.jsonl` are valid *pre-flags* traces and remain usable -
they are byte-for-byte the same field set as v5 - so nothing is lost, but the session-state work
(`switches_<room>`, `Session.DoNotLoad`, `Session.Cassette`) still needs one more export pass with the
v6 DLL installed.

## Round 52-54: the flags-carrying traces exist, and how to export them

`trace-202-v6b.jsonl` (1,798,002,573 B) and `trace-100pct-v6b.jsonl` (1,101,479,186 B) were exported with
the v6 exporter and **asserted**: the first 40,000 rows of the 202 file contain 36,739 with a `flags`
array (`"flags":["CelesteTAS_TAS_Was_Run"]` immediately after `inventory`, in the append-only tail).

Recipe, including the two traps that cost a round each:

1. install the v6 DLL (from `celestetas-trace-w8v5`, SHA256 `DDAC7E16...33F4739`) over
   `game-trace/Mods/CelesteTAS-EverestInterop/bin`, then **delete `game-trace/Mods/Cache/CelesteTAS*`**
   so Everest rebuilds the merged cache from the new DLL - otherwise the loader keeps serving the old one
   and the run reports `status: success` with a pre-flags trace;
2. pass the **plain** TAS as `-TasFile` (`0 - 100%.tas`, `0 - 202 Berries.tas`), never the `_trace-*.tas`
   helpers: those carry their own `TasFrameTrace` line, and the second command's `Begin()` flushes the
   first (empty) writer to 0 bytes and redirects every row to the *other* file;
3. **assert the new key before believing the trace** - size, wall time and `status: success` were all
   normal on the export that turned out to carry no `flags` at all.

Not yet done: the gate has not been run against the v6b traces. Expect per-segment identity with the v5
baselines (the gate ignores unknown keys), but `crates/` has been owned by the `dashSwitchH/V` workstream
since round 47, so the run is queued behind it - measuring then would only measure that half-finished
tree.

### The v6b trace is validated and now canonical

`trace-202-v6b.jsonl` (which carries `flags`) replays to `511 ok / 159,351 frames / 158,364 exact`
against the v5 baseline's `511 / 159,350 / 158,363`: `improved=1, identical=1467, regressed=0`. The
one-frame difference is the game's own run-to-run variance, not the new key - the gate ignores unknown
fields, so the added array cannot perturb it. The v6b traces therefore supersede v5 as ground truth, and
they are the ones to read `flags` from.

### Consuming `flags`: the exact three steps (one attempt failed here and was reverted)

The v6 trace's `flags` array reaches the harness but the anchor restore cannot see it yet, because the
field has to cross one more layer than it looks:

1. `struct Record` (`examples/tas_fidelity.rs:185`) - the raw serde row. Add
   `flags: Option<Vec<String>>` and nothing else is needed; the key deserializes by name.
2. `struct Frame` (`:252`) - the parsed per-frame view that `segment.frames[..]` holds, and *that* is
   what the anchor is (`let anchor = &segment.frames[window_start]`). Add `flags: Option<Vec<String>>`
   here too, and copy it across wherever a `Frame` is built from a `Record`.
3. The anchor restore (`:1739`/`:1743`) - replace `set_clutter_cleared(carried_clutter)` and
   `set_switches_on(switch_room_flag(segment))` with the trace-derived values when the anchor row has
   the key, falling back to the carries otherwise: `"switches_" + segment.room` and
   `oshiro_clutter_cleared_0/1/2` are the two lookups.

Attempting only steps 1 and 3 fails with `error[E0609]: no field flags on type &Frame` (plus two
`E0282`s from the same site); the edit was reverted and the tree rebuilt, so the committed state stays
green. The two carries (`SWITCH_ROOM` at `:52-57`, `Y3_CHAPTER`/`Y3_CLUTTER` at `:60`) then become the
v5 fallback only - keep them, but say so in the comment, because v5 traces have no `flags` key and must
keep working.

### Session state now comes from the trace's `flags` (`16ee600`)

The anchor restore reads `switches_<room>` and `oshiro_clutter_cleared_0/1/2` out of the anchor row's
`flags` array and only falls back to the hand-rolled `SWITCH_ROOM` / `Y3_CHAPTER` / `Y3_CLUTTER` carries
when the key is absent (v5 traces). All three traces measured **regressed=0 and identical to the byte**:
202 `0/1468/0` (511 ok, 159,351 frames, 158,364 exact), 100pct `0/918/0`, 1a `0/20/0`, with 339 tests
green.

That "no change at all" is itself the measurement: on this corpus the guessed carries agreed with the
trace everywhere they mattered. What changes is what happens where they *could not* agree - the
documented `2-OldSite|0|6` case, whose `switchGate` is `persistent=false` so the flag is never written,
now reads "not set" from ground truth instead of from a guess.

Two notes for the next editor: the field has to be added to **both** `Record` and `Frame` and copied in
the `segment.frames.push(Frame {` construction (the anchor is a `&Frame`, not a `&Record` - that is what
made the first attempt fail with `E0609`), and struct headers plus that `push` line are the only anchors
that are unique: `collider:` exists on `Record` alone and `dt: record.dt,` also matches other
construction sites.

### Cross-trace diffs mix in the game's own +-1-frame variance

`trace-202-v7.jsonl` (which adds `doNotLoad`/`keys`/`cassette`/`heartGem`) replays to
`511 ok / 159,350 frames / 158,363 exact` against v6b's `511 / 159,351 / 158,363`:
`improved=0, identical=1467, regressed=1`, the single difference being `7-Summit|0|d-11|120868` losing
one frame (698 -> 697). **No code changed between those two runs** - same binary, different ground
truth - so that frame is the real game's run-to-run variance, exactly as the v5-vs-v6b comparison
showed one segment *gaining* a frame. The four new keys themselves are inert for the replay, which is
what the check was for.

The consequence for the usual discipline: `q-regress` labels are relative to the trace it was given,
so a one-frame `regressed` entry that appears when the trace changes (rather than when the code does)
is noise, not a regression. When a code change is measured, keep the trace fixed and swap only the
binary; when the trace changes, expect about one frame of drift somewhere and do not chase it.

### `doNotLoad` needs the map id: use a parallel vector, not a new `Entity` field

`Session.DoNotLoad` holds `EntityID.Key` strings (`"<Level>:<ID>"`), so consuming it means matching the
map's `id` attribute - and `pub struct Entity` (`crates/celeste-physics/src/map.rs:191`) does **not**
carry it. The obvious fix is a new field, but that is a trap: `map.rs` has **31 `Entity { ... }`
construction sites**, so every one of them would have to be touched, in a file where a single missed
literal is a compile error and my last attempt at a multi-site edit had to be reverted.

The cheap route: keep `Entity` untouched and add a **parallel `Vec<i32>` on `Map`** (e.g.
`entity_ids`), filled in the same push order as `entities`, from `attr_f32(el, "id", <current
synthesis>)` at the one decode push site - the encoder already synthesises `index + spawns.len()`, so
that stays the default and no existing behaviour changes. Then `doNotLoad` membership is
`entity_ids[i]` formatted as `"<level>:<id>"`, and `Simulator::new` can skip those entities (which is
what `Level.cs:472`/`:1188` do - the engine's only construction-suppression point).

Two more things that belong to the same change: `conditionBlock condition:Key` and `ridgeGate` are
*conditional solids* whose existence is decided by `DoNotLoad` membership, and in opposite directions -
`conditionBlock` adds an `ExitBlock` only when the referenced key **is** in the set (`Level.cs:863` +
`:868`), `ridgeGate` only when all its listed keys are (`:878-883`, `:1475-1482`). And the same
parallel-vector trick applies to `Session.Keys` for `LockBlock` (`LockBlock.cs:88-98`).

### Step 1 of `doNotLoad` is cheap: `Map` has three literal sites, not thirty-one

A naive count of `Map {` in `map.rs` returns 31 and looks alarming, but it is a false positive - the
pattern also matches `HashMap {`. The real sites are:

- `impl Default for Map` at `:284` (so a new `#[serde(default)] pub entity_ids: Vec<i32>` needs a line
  there, or comes free if the impl is derived),
- `Map {` literals at `:341`, `:1535` and `:2243` (the last two are the ones to check for exhaustive
  field lists),
- the decode push at `:2036` (`entities.push(Entity {`), which is the single place the id attribute has
  to be read.

Compare with `Entity`, which genuinely has ~31 `Entity { ... }` sites and is why the field was not added
there. So the parallel-vector route is a handful of lines in one file, and the remaining work
(`doNotLoad` through `Record` -> `Frame` -> the anchor, then skipping entities in `Simulator::new`) uses
the anchor path that `flags` already proved out.

### `entity_ids`, attempt 2: the exact sites (attempt 1 is fully diagnosed)

Attempt 1 failed harmlessly (the guards aborted the write on the first try; the second try compiled
wrong and was reverted, tree rebuilt). Everything it got wrong is now known:

- The decode push is **`map.entities.push(Entity {`** at `map.rs:2036` - the receiver is `map.entities`,
  *not* a local `entities`; at `map.rs:2044` and around, the name `entities` is the `BinaryElement`
  holding the map's entity list, which is why inserting `entities.push(...)` there failed with
  `E0599: no method named push for &BinaryElement`. `el` **is** in scope at `:2036` (it is used at
  `:2028` and `:2041`), and the loop binding is `for el in &entities.children` at `:1555`, inside
  `fn map_from_binary_inner` (`:1510`).
- There is a **second** push at `map.rs:2083` in the same function. The parallel vector must stay in
  lockstep with `map.entities`, so either push an id there too or assert
  `entity_ids.len() == entities.len()` after decoding - a desync would silently mis-attribute every
  `doNotLoad` key after it.
- `Map` literals outside `map.rs` also need the field (or `..Map::default()`): **`map_fixture.rs:155`**
  and **`playground.rs:8`**. The two in `map.rs` at `:341` and `:2243` already use `..Map::default()`.
- The struct field goes on `Map` (`pub struct Map {` at `:235`) and the default in the manual
  `impl Default for Map` (`:284`, `Self { ... }` at `:286`) - adding it to `Entity` instead means
  touching ~31 `Entity { ... }` sites, which is the trap this route avoids.

### `entity_ids`, attempt 3: one site has no `el`, and PowerShell anchors must match CRLF

Attempts 2 and 3 both ended in the auto-revert guard (build failed -> `git checkout` -> rebuild), so the
tree never stayed broken; total cost about two minutes each. What they learned:

- The first push, `map.rs:2036`, is `map.entities.push(Entity {` and **does** have `el` in scope, so
  `map.entity_ids.push(attr_f32(el, "id", -1.0) as i32);` in front of it compiles.
- The second push, `map.rs:2083`, reaches the same `map.entities.push(Entity {` text but is in a branch
  where **`el` is not bound** - a blanket `String.Replace` on that text hits both and fails with
  `E0425: cannot find value el in this scope`. Either give that branch its own id expression, push a
  sentinel there, or push ids from a single place after the loop; and then assert
  `entity_ids.len() == entities.len()`, because a desync silently mis-attributes every later
  `doNotLoad` key.
- PowerShell `[char]10` anchors do **not** match this file: it is CRLF. Multi-line anchors built with
  `[char]10` silently fail (the guard catches it), so either anchor on a single unique line - e.g.
  `bounds: Rect::new(0.0, 0.0, 960.0, 544.0),` in `playground.rs` - or build the newline as
  `[char]13 + [char]10`.
- The other two files' literals are exhaustive: `map_fixture.rs:155` (`let map = Map {`) and
  `playground.rs:8` (`Map {` under `pub fn mechanics_playground() -> Map {`) each need
  `entity_ids: Vec::new(),` added; the two in `map.rs` already use `..Map::default()`.

### `entity_ids`, attempt 4: there are THREE `map.entities.push(Entity {` sites, not two

`Select-String` reported pushes at 1189, 2036 and 2083; a "replace the first, then the next" strategy
therefore patched 1189 and 2036 and failed with `E0425: cannot find value el` **and**
`cannot find value trigger` in one build. `el` is bound only inside `map_from_binary_inner`'s entity
loop (`for el in &entities.children`, `:1555`), and `trigger` only in the wind-trigger branch, so the
1189 site has neither. The auto-revert guard ran again (build failed -> checkout -> rebuild in 1m39).

The reliable shape for this edit is a regex that anchors on the **following line**, with `\r?\n` rather
than a bare LF, because the file is CRLF:

- site A (`:2036`, `el` in scope):
  `map\.entities\.push\(Entity \{\r?\n(\s+)kind,` -> `map.entity_ids.push(attr_f32(el, "id", -1.0) as i32);\r\n$1map.entities.push(Entity {\r\n$1kind,`
- site B (`:2083`, wind trigger, `trigger` in scope):
  `map\.entities\.push\(Entity \{\r?\n(\s+)kind: EntityKind::Wind,` -> the same with `trigger`.

The following-line discriminator plus `\r?\n` is what makes it unique; plain string replaces and
`[char]10` anchors both failed here. Everything else in the change was already proven to compile: the
`Map` struct field, the manual `Default` (`:286`), and the two exhaustive literals in
`map_fixture.rs:155` and `playground.rs:8`.

### `dashSwitch_<room>:<id>` is now consumed, and it is inert on all three traces (measured)

`persistent` was already decoded (`804cb39`, `map.rs:2058` carries it in `single_use`, and
`celeste_dash_switches_decode_with_source_colliders` asserts the round-trip); what was missing was the
consumer. `DashSwitch.Awake` (`DashSwitch.cs:124-149`) short-circuits when the session flag is set:
`Position = pressedTarget - pressDirection * 2f`, `pressed = true`, `Collidable = false`. That is the
state `OnDashed` itself leaves behind (`:203-205`), so the switch is simply *not a Solid* at the
anchor, which is what `park_entity` models for every other collidable-flag Solid. The new
`Simulator::set_pressed_dash_switches(ids)` (the harness filters `"dashSwitch_<room>:"` out of the
anchor row's `flags`, the same way it already reads `switches_<room>` and
`oshiro_clutter_cleared_*`) parks exactly the entities whose `map.entity_ids[i]` is named *and* whose
`single_use` (i.e. `persistent`) is set.

**Measured: `202 0 improved / 1468 identical / 0 regressed`, `100pct 0 / 918 / 0`, `1a 0 / 20 / 0`,
with every total byte-identical to the baselines (`511 / 159,350 / 158,363`, `334 / 95,613 / 95,012`,
`16 / 2,129 / 2,125`), 341 lib tests green.** The flags are real (9 distinct keys, 135,590 rows of
`trace-202-v7`: `b-00:15`, `a-00:16`, `b-11:15`, `b-13:56`, `a-08:12`, `b-08:231`, `d-15:218`,
`a-12:288`, `a-05:30`; 1a's v5 trace has no `flags` key at all), and the ids were checked against the
real `.bin` attributes. The restore is *not* dead code: an env-gated print showed it firing on 9
segments of the 202 trace (`a-12|60158`, `a-12|60680`, `a-08|60869`, `a-05|63301`, `b-13|68402`,
`b-11|68493`, `b-11|68702`, `d-15|78627`, `d-15|78905`), and at `d-15` only id 218 of the room's
218/219 pair is named by the flag. So the mechanism runs and moves nothing: in those windows the
already-pressed button's old 8x16/16x8 box is never probed by the player's collision, and a press
that happens *inside* a window was already handled by `on_dash_collide`. Landed as a faithful
transcription plus two real unit tests, exactly like the `CanUnDuck` round.

Two things it deliberately does not do, so nobody "adds" them later:

* **The gate half of `Awake` is a no-op here, not a missing feature to invent.** `StartOpen`
  (`TempleGate.cs:146-151`) is `SetHeight(0); open = true`, and `initialize_temple_gates` already
  starts *every* gate at `current_height == 0` / `open == true`; the simulator's only gate transition
  is `close_temple_gate`. `TempleGate.SwitchOpen`'s 0.4 s alarm before the collider collapses
  (`:124-132`) is **not implemented at all**, so it is not claimed here - and `allGates` is not
  decoded, because it only chooses *which* gates `Awake` opens. The real gap runs the other way: a
  `NearestSwitch` gate that is still closed in the game is already open in the simulator, because the
  gate `type` is not decoded either.
* **A v5 trace has no anchor restore for a dash switch.** `flags` is the only witness of a press that
  happened before the window, and nothing carries it the way `SWITCH_ROOM` carries
  `switches_<room>` for v5. On the current gates that costs nothing (1a has no dash switch, and
  202/100pct are v7).

### The wind ordering is NOT the lever: three placements, all net-negative (measured)

The recon behind this was strong - gaps of exactly `wind * 0.1 * dt` on 12 + 2 + 6 segments of the three
largest mismatch classes, with the guards (`Ducking && onGround`, `speed.y < 0 || !grounded`,
`state != Dash`) provably reading the previous frame's values. Three placements of
`apply_wind_movement` have now been measured against a fixed trace:

| placement | 202 result |
| --- | --- |
| before the `Player.Update` mirror (today) | baseline |
| after the state callback (round 45) | 58 improved / 22 regressed, lost an `ok` in `7-Summit\|1\|e-00` |
| after the ground probe, before the callback (round 91) | 4 improved / 11 regressed, lost the same `ok` |

The third was the narrowest reading of the evidence - it fixes exactly the `Ducking && onGround` guard
and leaves `State`/`Speed.Y` alone - and it still loses. So the stale guards are real but something else
in the frame order compensates for them, and **moving the application is not the fix**. Do not try a
fourth placement without first finding that compensating site; a profitable attack on these classes has
to start from a segment where the wind term appears *and* the moved placement changes nothing, then ask
what else consumed the difference.

### The `2-OldSite|0|6` one-frame divergence is NOT the gate slide timing

Two source-grounded fixes to the `SwitchGate` phase machine were measured on that room and changed
nothing (`135 frames / 134 exact` in all three runs):

1. the slide's one-frame priming (`phase 3 -> 4` set `timer = dt`, which starts the tween a frame early
   versus `Tween.Create(..., start: true)` being updated on the following frame) - removing it measured
   identical;
2. the four phase transitions falling through into the next phase's `timer -= dt` **in the same frame**,
   which makes the whole sequence about four frames early (with `Ease.CubeOut` at ~1 px/frame early on,
   that is exactly the "simulator has slid >= 4 px while the game is still at 0" the recon inferred) -
   adding `continue;` to all four measured identical too.

So the earlier claim in this document that the kept regression is a gate-slide timing difference is
**not supported**: neither timing fix moves those two segments. The geometry that motivated it was
inferred from the player's pose, not observed, and the divergence sits on the segment's last frame.
What is established is that the room's `switchGate` is `persistent=false`, so its session flag is never
written, and that both timing fixes are no-ops for the corpus. The next attempt should dump the gate's
per-frame position *and* the player's collider side by side for that window instead of inferring one
from the other.

### Two things off the critical path (recorded so they are not re-litigated)

**The `DoNotLoad` workstream is corpus-inert, and so are its conditional solids.** Parking the entities
`Session.DoNotLoad` names measured `0 improved / 1468 identical / 0 regressed` on 202 (commit `822134a`),
and the two *conditional solids* it decides are the same story: `Level.cs:853-871` builds an `ExitBlock`
for a `conditionBlock` only when its `conditionID` (`"Level:ID"`, `:857-859`) satisfies the condition -
`Session.DoNotLoad.Contains(id)` for `Key` (`:863`, the default), `Session.GetFlag(DashSwitch.GetFlagName(id))`
for `Button` (`:862`) - and `ridgeGate` (`:878-883`) is built only when `GotCollectables` holds. Both
conditions are now knowable in the simulator (the do-not-load set and the pressed-switch flags are both
carried), but `conditionBlock` appears in exactly one map, so this is not where the remaining 956
mismatches live. Left unimplemented on purpose.

**Heavy gate runs must be serialised.** Three concurrent 202 gate runs (each reading the 1.8 GB trace)
made one of them exit `1` with **no output at all** - which reads like a crash and is not one: a
`--limit-segments 60` run of the same binary immediately succeeded. If a gate run dies silently, check
for concurrent runs before suspecting the change. Sub-200-segment runs (`--rooms`, `--limit-segments`)
are the right tool for a single hypothesis and cost ~20 s.

### Builds contend too, not just gate runs

The serialisation rule has a second half. Concurrent work (in this case two subagents working in their own
worktrees) makes `cargo build` fail with `error: linking with link.exe failed: exit code: 1104` - the
linker cannot replace the output while another link is in flight. The tree is fine and the binary on disk
is simply the last one that linked, so the failure is *cosmetic but misleading*: a probe added at the same
time then runs as the **old** binary and prints nothing, which reads exactly like "the code path never
executes". That is how a gate probe appeared to show zero output this round.

Practical rule: when several things are running, build when nothing else is; never interpret a silent
diagnostic as evidence until a build has actually succeeded (`Finished` in the output, not just a return).

### Correction: `Level.EnforceBounds` IS modelled (the bullet above is stale)

The note above says `Level.EnforceBounds` is not modelled. It is: `sim.rs::enforce_level_bounds`
(`sim.rs:10148`) gates on exactly the source's condition (`dead || DreamDash || !in control`, matching
`Player.cs:1915-1918`), clamps the collider to `current_room_bounds` (falling back to `map.bounds`),
zeroes `speed.x` on a clamp, and performs the four-direction transition hand-off with the same
`player.Center +/- 8` probe (`Level.cs:2725-2790`) via `transition_room_at` + `begin_transition`. So
anyone reading the bullet above would re-implement something that already exists. Left as a correction
rather than an edit so the original claim and its rebuttal stay visible.

### The dash publish must be DELAYED one frame, not skipped (a failed fix, measured)

The recon's instance is real in shape: on the DashBegin frame the trace shows `Speed=[0,0]`,
`DashDir=[0,0]` and a byte-identical `Position`/`movementCounter` (the game is inside
`Celeste.Freeze(0.05f)`, `Player.cs:4284`), while the simulator publishes `dash_dir = last_aim;
speed = dash_dir * DASH_SPEED` on that same frame (`dash_update`'s publish block, guarded only by
`(state_timer - DASH_TIME).abs() <= dt*0.5`). 36 of the 133 `pos|StDash` segments have that shape.

But the obvious fix is wrong, and measurably so: adding `!dash_coroutine_initial_yield` to the publish
guard (i.e. **skipping** the publish on the begin frame) measured **0 improved / 300 identical / 1168
regressed**, `511 -> 40` ok, `159,350 -> 46,418` frames. The reason is mechanical: that guard is true
only on the begin frame and the window (`|state_timer - DASH_TIME| <= dt*0.5`) is *also* only satisfied
on that frame, because `begin_dash` sets `state_timer = DASH_TIME + dt` and `dash_update` subtracts one
`dt` before the check. Skipping therefore drops the publish entirely and no dash ever starts.

So the correct change is to **delay** the publish by one frame - keep the write, but make it happen on
the frame after `DashBegin`, i.e. widen the window by one `dt` on the low side while keeping the
initial-yield term out of the way - and to check `DashDir`/`Speed` against the trace on both frames
rather than just the end of the begin frame. Reverted; tree rebuilt.

### The dash publish needs a one-frame pending flag, not a moved window (second failed fix)

Delaying the whole publish block by one frame - `if p.state_timer <= DASH_TIME - dt*0.5` in place of the
symmetric window - measured **2 improved / 261 identical / 1205 regressed** (`516 -> 37` ok,
`159,915 -> 43,395` frames). Together with the previous attempt (skipping the publish: 0/300/1168) that
brackets the defect precisely:

- the block *after* the guard is not just the publish - it carries the dash's own bookkeeping, which must
  keep running on the `DashBegin` frame (moving it breaks every dash);
- the publish itself (`dash_dir = last_aim; speed = dash_dir * DASH_SPEED`, plus the before-dash-speed
  carry and the water multiplier) must happen on the frame *after* `DashBegin`, when `DashCoroutine`'s
  initial `yield return null` resumes;
- skipping it entirely never publishes, because the guard `|state_timer - DASH_TIME| <= dt*0.5` is
  satisfied only on the begin frame (`begin_dash` sets `state_timer = DASH_TIME + dt` and one `dt` is
  subtracted before the check).

So the shape that can work is an explicit pending flag: on the begin frame set "publish due" and do not
write, on the next frame publish and clear. That is a `PlayerSnapshot` field the trace does not export
(derived, like `previous_state`), so it also needs an entry in the field-coverage table - which is
exactly what the `--dump-field-map` audit has been warning about.

### Third dash attempt: even delaying only the two publish lines breaks 310 segments

The `dash_publish_pending` shape - set the flag on the begin frame, write `dash_dir`/`speed` on the next,
leaving everything else in place (the before-dash carry and the water multiplier are no-ops while
`speed` is zero) - measured **ok 516 -> 206** on 202. So the boundary is tighter than "the two lines":
some other part of the same frame's dash accounting is load-bearing in a way the trace's per-row view
does not show.

Three measured failures now bracket this (skipping: 1168 regressed; whole-block delay: 1205; two-line
delay: 310), and the third says the next step is **not** another edit to the publish but a per-frame
comparison. Concretely: the recon's own suggestion - put `movementCounter` into the gate's compared
fields and print it at `{:.9}` (the dump's `{:.5}` throws away exactly the quantity at issue) - then look
at the *sequence* of frames around one of the 36 `pos|StDash` DashBegin rows (e.g. `4-GoldenRidge|0|d-01`
row 303301) on **both** sides: how many frames the game's freeze lasts, which frame first moves, and
which frame the sim first moves. Only after that is it worth touching the publish again.

### The game's dash frames, measured row by row (`4-GoldenRidge|0|d-01`, rows 303300-303305)

| row | state | collider | dashAttackTimer |
| --- | --- | --- | --- |
| 303300 | StNormal | `[9713,-4400,8,11]` | 0 |
| **303301** | **StDash** | `[9713,-4395,8,6]` | **0.3** |
| 303302 | StDash | `[9713,-4395,8,6]` | 0.3 |
| 303303 | StDash | `[9713,-4395,8,6]` | 0.3 |
| 303304 | StDash | `[9713,-4395,8,6]` | 0.3 |
| **303305** | StDash | `[9718,-4397,8,11]` | 0.28333 (= 0.3 - dt) |

Read it precisely: row 303301 keeps the same collider *bottom* (-4400+11 = -4395+6 = -4389), so that
frame only swaps to the ducking collider - the move is **zero** - and `DashBegin` has already set
`dashAttackTimer = 0.3` and `DashDir`/`Speed` to zero. Rows 303302-303304 are byte-identical with the
timer **frozen**: three frames of `Celeste.Freeze(0.05f)`. Row 303305 is the first frame that moves and
the first on which the timer decrements, i.e. **the publish happens on the first DashUpdate that
actually executes after the freeze**, not one frame after `DashBegin`.

That means the three failed edits were all reasoning about the wrong frame count. Before touching the
publish again, measure the simulator's own freeze length for the same window - how many frames it skips
`dash_update` for, and which frame it first moves on. If the simulator freezes for a different number of
frames, the publish timing is a *symptom* of that, and the fix belongs in the freeze accounting, not in
`dash_update`. A `--rooms d-01` slice with a per-frame counter print answers it in seconds.

### The simulator's freeze accounting already matches the game - so stop editing the publish

`sim.rs:6314-6317`: during a freeze the simulator decrements `freeze_timer` by `DT` and **returns early**,
so the whole player update - `dash_update` included - is skipped for exactly as many frames as the timer
lasts. `begin_dash` sets `DASH_FREEZE_TIME` (`sim.rs:7149`), 0.05 s, which is 0.05/DT = **three** skipped
frames, and the measured game rows 303302-303304 are exactly three byte-identical frames with
`dashAttackTimer` frozen at 0.3. The `DashBegin` frame itself runs the state transition (Monocle runs the
new state's callback on the frame after the change), so the first `DashUpdate` - and therefore the
publish - lands on the first frame after the freeze, which is what the game does.

Consequence: the three failed edits were not fixing a publish-timing bug, because there is probably no
publish-timing bug. What the recon saw (`rustMove = -1.33334` on the row it read as the begin frame)
should be re-checked against the *row mapping* first: the gate compares its frame `j` with row
`startRow + 41 + j`, and a one-row slip would put the publish on the "wrong" row without anything being
wrong in `dash_update`. The cheap way is the recon's own suggestion - compare `movementCounter` at
`{:.9}`, and print the row index alongside - across rows 303299-303306 on both sides. Only if the rows
line up and the counters still disagree is the publish implicated.

### SOLVED: the pos|StDash DashBegin deltas ARE the wind, applied one guard-state too early

The row-aligned dump settles it. `4-GoldenRidge|0|d-01` at the DashBegin row (303301, dump offset 1):
game `gameMove=(0,0)`, `gameSpeed=(0,0)`, position unchanged; simulator `rustMove=(-1.33334, 0)` with the
same end-of-frame `(0,0)` speed. The mapping is correct (`offset 0` is row 303300, matching the trace), so
this is not an alignment artefact.

`-1.33334 = -800 * 0.1 * dt` - that is the **wind** term, not the dash (a down-right dash would be about
`314 * 0.707` per axis). The game does not push it because on that frame the state is already `StDash` and
`Player.WindMove`'s guard excludes `StBoost`/`StDash`/`StSummitLaunch`; the simulator applies the wind
**before** the state callback, so its guard still sees `StNormal` and the push lands. So the largest
`pos|StDash` class and the wind-ordering lead from round 45 are **one root cause**, and the earlier
"publish the dash velocity one frame early" reading was wrong: the extra movement is wind.

What this changes for the fix: the round-45 experiment moved the whole `apply_wind_movement` call after
the dispatch and measured 58 improved / 22 regressed - i.e. it fixed exactly this guard but broke the
other two (`Ducking && onGround`, `speed.y < 0 || !grounded`). The next attempt must move the call and
then verify **all three** guards against the trace, one at a time, on the specific rooms that regressed
(7-Summit and 4-GoldenRidge): for each, the question is whether the game's value is the start-of-frame
one (`onGround`, computed before `base.Update()` at `Player.cs:1504-1526`) or the post-callback one
(`Speed.Y`, `Ducking`, `State`). Only the state guard demonstrably needs the later position.

### The wind has two populations: a global move cannot work, the guard has to be per frame

Moving `apply_wind_movement` after the state callback measured **4 improved / 11 regressed** again, and
the regressing set is *the same* as the round-91 attempt and a subset of round 45's: `4-GoldenRidge`
`a-02`, `c-00`, `c-05` and `7-Summit` `e-05`, `g-00b`, `g-00`, `e-00` - the windy chapters, exactly. Their
frame counts *drop* (`g-00b` 289 -> 93, `g-00` 581 -> 299), i.e. they now diverge **earlier**, so for them
the early placement was the closer one.

Read together with the proven case (the dash-begin frame, where the game's state guard sees `StDash` and
skips the wind while the simulator pushes `-800*0.1*dt`), that means the two are different populations:
a global move of the call fixes one and breaks the other. What is needed is the *move* staying where it is
(the early placement is what those windy segments want) with the **state guard evaluated against the state
the callback ends the frame in** - i.e. a per-frame decision, not a re-ordering.

Three ways to get that, in increasing ugliness: pre-compute whether this frame's callback enters
`StDash`/`StBoost`/`StSummitLaunch` from the input and the dash conditions (exact but duplicates them);
apply the guard only for the state (compute the wind displacement early but publish it after the
dispatch, which is the "pending" shape that the dash experiments already showed is delicate); or apply the
wind early and reverse it after the dispatch when the new state is one of the three. The first is the
cheapest to reason about; whichever is chosen, it has to be measured against the same 11 segments.

### The wind fix, determined: apply early, undo when the callback ends in an excluded state

Put together with `sim.rs:10972` (`apply_wind_movement`), the shape is now fully specified.

The function's own guard (`!player_in_control || no_wind_timer > 0 || matches!(state, Boost | Dash |
SummitLaunch)`) is evaluated **before** the state callback, which is the defect: on a dash-begin frame the
game's callback has already set `StDash`, so `Player.WindMove` returns without pushing, while the simulator
pushes `wind * 0.1 * dt` (measured: `-800 * 0.1 * dt = -1.33334` on `4-GoldenRidge|0|d-01` row 303301).
But the *move* must stay where it is, because the wind-chapter segments (`4-GoldenRidge` `a-02`/`c-00`/
`c-05`, `7-Summit` `e-05`/`g-00b`/`g-00`/`e-00`) diverge **earlier** when it is moved later.

So: keep the early call, and after the dispatch undo it exactly when the new state is one of the three.
Two mechanical details that are easy to get wrong:

1. **Do not restore an absolute `(pos, movement_remainder)` snapshot.** Between the wind call and the
   dispatch the state callback can move the player itself, and an absolute restore would throw that away.
   Return the *applied* amounts from `apply_wind_movement` (`-> (f32, f32)`, `(0.0, 0.0)` on every early
   return) and, if the post-dispatch state is `Dash`/`RedDash`/`Boost`/`SummitLaunch`, call
   `move_axis_amount(p, map, true, -move_x)` and `move_axis_amount(p, map, false, -move_y)`. The wind was
   applied into space the player just came from, so the inverse move cannot collide and the counter
   restores with it.
2. **The two other guards stay as they are.** The code's own comment already records why
   `p.ducking && p.player_on_ground` uses `player_on_ground` (the previous frame's probe) rather than the
   geometric `on_ground` - that one was fixed deliberately and must not be "corrected" while doing this.

Verify against the same 11 segments plus the dash-begin row, and expect the frame counts of those 11 to
stay at their baseline values.

### The wind undo fires and works on the dash frames (36 improved) but wrecks the windy segments (17 regressed)

Measured with the early apply kept and the delta undone after the dispatch when the new state is
`Dash`/`RedDash`/`Boost`/`SummitLaunch`: **36 improved / 1415 identical / 17 regressed** (`516 -> 515` ok,
`159,915 -> 156,345` frames). That is the first wind variant that fixes a real number of segments - the
dash-begin population from the `d-01` row 303301 measurement is genuine - but the windy segments collapse
(`7-Summit|0|g-01` 833 -> 75, `g-00` 581 -> 101, `7-Summit|1|e-00` loses its `ok`), and now `d-00`,
`e-13` and `LostLevels|0|h-05` regress too.

Both populations are therefore real and the condition as written does not separate them. The natural
reading: in those segments the simulator is already in `StDash` on frames where the game still applies
wind, i.e. the sim's **state transition** is ahead of the game's, and the undo then removes a push the
game made. That would make the windy-segment problem a state-*timing* problem upstream of the wind rather
than a wind-placement problem - which also fits the earlier variants: moving the whole call later changed
the same set of segments, in the same direction.

Next diagnostic, cheap: keep the undo and dump `7-Summit|0|g-01|134447` (833 -> 75, the sharpest drop)
side by side with the trace, and read the frames where the undo fires - what the game's state and wind
displacement are on each. The answer decides whether the fix belongs in the wind at all.

### Measured: the `7-Summit|0|g-01` population is an updraft applied in `StNormal`, not a dash

Rows 134555-134560 of `trace-202-v7` (`7-Summit|0|g-01`, segment start 134447, so these are replay frames
~64-69): the player is `StNormal`, jumping, `spd=(0,-105)`, `wind=(0,-400)` and `windTarget=(0,-400)`.
The per-frame y deltas are -2, -3, -2, -2, i.e. **2.417 px/frame = 105/60 (the jump) + 400*0.1*dt (the
wind)**. The game is applying a vertical updraft to a normally-jumping player here.

That matters because the undo variant only fires in `Dash`/`RedDash`/`Boost`/`SummitLaunch`. For it to
damage this segment, the *simulator* must be in one of those four states on frames where the game is
plain `StNormal` - or the guard's exclusion set must not be the game's. The baseline replays this segment
for 833 frames, so the two state machines agree there in the baseline; therefore the interesting question
is narrower: **which state does the simulator have on the frame the undo first fires, and what does the
gate's own dump say `gameState`/`rustState` are on that frame?**

The exact command (baseline binary, no change needed):
`.\target\release\examples\tas_fidelity.exe --trace D:\celeste-research\.tmp\tasrun\trace-202-v7.jsonl --maps vendor\celeste-game\Content\Maps --out <report> --rooms g-01 --dump-segment "Celeste/7-Summit|0|g-01|134447"`
and read the lines whose `rustState` is one of the four (Boost/Dash/RedDash/SummitLaunch) plus the rows
around them. If `gameState == rustState` on those frames, then the simulator's guard set is simply not
the game's, and `Player.WindMove`'s condition should be re-read from the source rather than assumed.

### The exact guard set changes nothing - the undo is not the windy segments' problem

Re-running the wind undo with the game's *precise* exclusion set - `StDash`(2) | `StBoost`(4) |
`StSummitLaunch`(10), read from `Player.cs:3085` and the constants at `:349-399`, so `StRedDash`(5) is
correctly **not** excluded - measured the same `36 improved / 15 identical-ish / 17 regressed` with the
same segments and the same frame counts as the variant that also excluded `RedDash`. So `RedDash` was not
the cause, and on those frames the simulator is in `Dash`/`Boost`/`SummitLaunch` exactly where the game's
guard also excludes wind, yet the game's movement there includes a wind term (measured on
`7-Summit|0|g-01`: 2.417 px/frame = jump 1.75 + wind 0.667 in `StNormal`).

That is a contradiction only if the two state machines agree at the frames in question, and the baseline
replaying 833 frames of that segment says they do at the *end* of each frame. The remaining possibility is
that they disagree *within* the frame - the callback's state at the moment `WindMover` would run - which
the gate's end-of-frame `gameState`/`rustState` cannot show. Settling it needs the wind decision printed
per frame (state at the wind call, after the dispatch, and the delta) on a slice of `g-01`, not another
global edit.

### Merging `floaty` needs four test call sites updated (`Simulator::new` now takes `&mut Map`)

The `floaty` branch (`187c7b9`, 7 improved / 0 regressed on both v7 traces, ok 511 -> 513 and 334 -> 336,
344 tests) changes `Simulator::new` to take `&mut Map`, because its `Awake` pre-roll moves the blocks
before the first update. Merging it onto current `master` therefore breaks the *test* build with four
`E0308` mismatched types (expected `&mut map::Map`, found `&map::Map`) plus two `E0596` "cannot borrow
`map` as mutable" in the tests added by the `gate`/`dashflag`/`cassette` branches.

A first mechanical pass fixed the `&ident` forms (`Simulator::new(x, &map)` -> `&mut map`) and added `mut`
to the bindings the compiler named, which cleared the `E0596`s, but four sites remain - they do **not** use
the `&`-prefixed form, so the exact lines have to be read from the compiler before patching. The merge was
rolled back rather than committed with a failing test build; the branch is untouched and can be merged
again as soon as those four sites are updated. Recorded so the next attempt starts from the error list
rather than from a regex.

## Baselines after `TempleGate` + `FloatySpaceBlock` (rounds 97-108)

| trace | ok | mismatch | frames | exact |
| --- | ---: | ---: | ---: | ---: |
| `trace-202-v7` | **518** | 950 | **160,183** | **159,203** |
| `trace-100pct-v7` | **339** | 579 | **96,228** | **95,632** |
| `trace-1a-v5` | 16 | 4 | 2,129 | 2,125 |

Three mechanisms landed with zero per-segment regressions, in this order: `SwitchGate` + `TouchSwitch`
(12 improved, plus the documented 1-frame exception), `dashSwitchH`/`dashSwitchV` under their real names
(+943 frames), then `TempleGate` (type, `Awake` start state, the `SwitchOpen` alarm and switch claiming;
516) and `FloatySpaceBlock` (real motion plus the derived `System.Random` phase; 518). `Session.DoNotLoad`
parking, the persistent dash-switch flag and `Session.Cassette` are in as well and measure inert on this
corpus, with the reasons recorded above.

### The `Speed.X` restore hypothesis has a concrete frame and a concrete gate

`7-Summit|1|e-02`, row 206422 (dump offset 1):

| | move.x | pos.x | end-of-frame speed.x |
| --- | ---: | ---: | ---: |
| game | **-0.98611** | 9796 -> 9795 | +10.83335 |
| simulator | **+0.18056** | 9796 (unmoved) | +10.83335 |

The end speeds match exactly, which is why the gate reports only `pos`. The game's in-frame value must
have been `-70 + 10.8333 = -59.1667`, i.e. it **restored `wallSpeedRetained = -70`** and then took one air
friction step; the simulator's `+10.8333` is pure air acceleration, so it never restored.

The gate is `sim.rs:7207-7218`, and the three branches are the reason it can skip: it drops the retention
when `math_sign(speed.x) == -math_sign(wall_speed_retained)`, restores when the probe
`map.solid_at(current_player_rect(p, pos.x + math_sign(wall_speed_retained), pos.y))` is **false**, and
otherwise just ticks the timer down. So on this frame the simulator either believes a solid is one pixel
in the retained direction or takes the "moving away" branch - while the game restored.

Next step is one print on that frame: `math_sign(p.speed.x)` vs `-math_sign(p.wall_speed_retained)`, the
retained value, the retention timer, and whether `map.solid_at(...)` is true, plus what the tile at that
pixel is. That distinguishes "wrong probe" from "wrong retained sign/value", and the same three-branch
block sits in every `pos|StNormal`/`pos+speed|StNormal` segment the recon sampled (`c-00` row 172768 is
the same shape), so it is worth resolving once.

### The wall-speed restore: the sim's retained value and probe both differ (frame-level measurement)

Printing the gate's inputs on the `7-Summit|1|e-02` slice (the print fires only while the timer is live,
so it fired on four frames):

```
pos=(9796,-8227) speedx=0.0000 retained=+165.6667 timer=0.0600 sign_speed=0 sign_ret=1 probe_solid=true
pos=(9796,-8230) speedx=0.0000 retained=+165.6667 timer=0.0433 sign_speed=0 sign_ret=1 probe_solid=true
pos=(9796,-8227) speedx=0.0000 retained=+207.6665 timer=0.0600 sign_speed=0 sign_ret=1 probe_solid=true
pos=(9796,-8230) speedx=0.0000 retained=+207.6665 timer=0.0433 sign_speed=0 sign_ret=1 probe_solid=true
```

Two things stand out:

1. **The retained value is positive** (+165.67 / +207.67, i.e. rightward) on frames where the game's
   in-frame speed arithmetic (`-70 + 10.8333 = -59.1667`) implies a **leftward** restore. So the sim's
   `wall_speed_retained` is not the game's, and the defect is **upstream of the gate** - in whatever wrote
   it (the wall-jump/retention writer, `Player.cs:1660-1676` region) or in how it decays.
2. **The probe says "solid" on all four frames**, so the sim never restores while the game does. Worth
   checking whether `current_player_rect(p, pos.x + sign, pos.y)` is the same rectangle the game's
   `CollideCheck<Solid>(Position + UnitX * sign)` tests - the source translates **`Position`** (the actor's
   top-left) whereas the helper may build the *hurtbox*, which is a different rect near a wall.
3. Also note `speedx = 0.0000` with `sign_speed = 0`: `math_sign(0)` is 0, so the first branch cannot fire
   here; the decision is entirely the probe.

Next: print the writer's assignment (the frame that sets `wall_speed_retained`) and the two rectangles
(the helper's and the raw `Position + sign` one) on the same slice. Reverted the probe; tree rebuilt.

### The wall-speed restore is innocent: the sim and the trace agree, and the missing thing is a -70 write

Reading the trace's own fields (`p.wallSpeedRetained`, `p.wallSpeedRetentionTimer` - the field map has them
at `p.*`, not top level) for `7-Summit|1|e-02` rows 206420-206423:

| row | retained | timer | end-of-frame speed.x |
| --- | ---: | ---: | ---: |
| 206420 | 165.6667 | 0.0600 | 0 |
| 206421 | 165.6667 | 0.0433 | 0 |
| **206422** | 165.6667 | **0.0267** | **+10.8334** |
| 206423 | 165.6667 | 0.0000 | +161.33 (a jump) |

The simulator's own probe printed `retained=+165.6667`, `timer=0.06` then `0.0433`, `probe_solid=true` -
i.e. **the same retained value, the same timer, and the same "blocked" branch the game takes** (the timer
ticks down on both sides). So the anchor restore and the gate are both correct, and the earlier suspicion
about the probe rectangle is moot.

What differs is the *move*: at row 206422 the game moved `-0.98611`, i.e. its in-frame speed was
`-59.1667`, while its end-of-frame speed is `+10.8334` - so a write of about `-70` happened during the
frame. `-70` is neither the exported `wallSpeedRetained` (+165.6667) nor `hopWaitXSpeed` (0), and the wind
in that room is `(0,-400)`, purely vertical, so it cannot supply an x component. The search therefore moves
from the restore gate to **whatever writes -70 during a `StNormal` frame near a wall** - candidates are the
wall-jump/hop family in `NormalUpdate` and any place that converts a retained value's sign.

### The missing -59.1667 on row 206422: what the trace does and does not contain

Full input/state sequence for `7-Summit|1|e-02` rows 206419-206423 (all `StNormal`, `mx=1`, `jump=true`):

| row | retained | timer | end speed.x |
| --- | ---: | ---: | ---: |
| 206419 | 130.0000 | 0.0100 | 0 |
| 206420 | **165.6667** (new) | 0.0600 | 0 |
| 206421 | 165.6667 | 0.0433 | 0 |
| **206422** | 165.6667 | 0.0267 | **+10.8334** |
| 206423 | 165.6667 | 0.0000 | **+161.3333** |

The simulator reproduces `retained` and `timer` exactly on every one of those rows, and it also lands the
wall jump's horizontal `161.3333` on row 206423. What it does not reproduce is row 206422's **move**:
`-0.98611` (in-frame speed about `-59.1667`) against the simulator's `+0.18056` (about `+10.833`). So on
that single frame the game carries a *leftward* speed that the simulator does not have, and the frame's
own end speed (`+10.8334`, identical on both sides) hides it - which is exactly the "one-frame writer
placement" shape the recon described for the large `pos` classes.

Two things are worth stating plainly. First, `-59.1667` is consistent with `-70 + 10.8333`, and **no
exported field in that window equals either -70 or -59.1667** (`retained` is +165.6667 / +130,
`hopWaitXSpeed` is 0, `speed.x` at row ends is 0 then +10.833), so the value is written and consumed
inside the frame. Second, this is the second consecutive round spent *locating* rather than fixing: the
wall-speed gate, the probe rectangle and the anchor restore have all been eliminated with numbers, and
what remains is a frame-internal write. A cheaper way forward than another single-row hunt is to ask the
question over a population - for every `pos`-class segment, dump the frame where the divergence starts and
check whether the game's in-frame speed (recovered as `move / dt`) equals a value the trace never reports
at a row boundary. If that is systematic, the missing writer is one code path, not one segment.

### Population check: "in-frame speed != row-boundary speed" is systematic

New tool `.tmp/recon3/in-frame-speed.mjs` (report + trace, no build): it walks the largest `mismatch`
segments and, for each frame, compares the game's move (`dpos / dt`) with the previous and the current
row's exported `Speed`, flagging frames that match neither within 2 px/s. First run over the eight largest
segments (`--rooms`-free, pure trace):

```
7-Summit|0|g-01  row=134575  prev=0.000    this=-170.000  dpos=-3  implied=-180.000
7-Summit|0|g-01  row=134576  prev=-170.000 this=-165.667  dpos=-3  implied=-180.000
7-Summit|0|g-00  row=131111  prev=0.000    this=-40.000   dpos=-1  implied=-60.000
7-Summit|0|b-09  row=110598  prev=316.000  this=316.000   dpos=8   implied=480.000
7-Summit|1|b-03  row=200810  prev=0.000    this=394.000   dpos=7   implied=420.000
```

Two caveats keep this indicative rather than exact: `dpos` alone is quantised to whole pixels (the
`movementCounter` is not exported), and the "implied" value therefore sits on a 60 px/s grid. Even so the
pattern is consistent - the frame's move corresponds to a speed one *writer step* away from the row
speeds (`-180` then `-170`, a friction step of 10.833 apart; `480` vs `316`; `420` vs `394`) - which is
the same "a `Speed` write lands on a different frame than in the game" shape the recon described, now
extractable in bulk.

The next step this enables is classification rather than another single-row hunt: for each flagged frame,
record the state pair and the delta between implied and reported speed, then group. If the deltas cluster
around friction (10.833), dash caps, or wall/hop values, the writer will name itself.

### The population tool failed its validity check - and that points at the exporter

`.tmp/recon3/in-frame-deltas.mjs` (report + trace, no build) classifies frames whose move-implied speed
matches neither the previous nor the current row's `Speed`, over the 40 largest `mismatch` segments:

```
scanned frames: 20507  flagged: 9516      <- 46% flagged
implied - previous speed, 5 px/s buckets:
  +10 x845  -30 x709  -10 x657  +30 x625  +5 x461  -15 x433  -5 x413  -25 x368  +25 x347 ...
  -170 x330                                <- the only cluster that looks like a signal
```

A 46% flag rate with a symmetric spread around small multiples of 5 is **quantisation noise**, not a
mechanism: `dpos` between two rows is whole pixels and the fractional part lives in
`Monocle.Actor.movementCounter`, which the trace does not carry. The only real cluster is `-170` (a
wall-jump/Summit-sized value), and single-segment dumping stays the only exact instrument - which is
exactly the slow route that has been eating rounds.

So the cheap enabling change is in the **exporter**: add `movementCounter` (and `ExactPosition`, if it is
cheap) to the player fields. Then every `pos`-class divergence can be classified quantitatively in one
streaming pass - `move = dpos + dcounter` is exact, `move/dt` is the frame's true in-frame speed, and the
delta against the row's `Speed` names the missing writer - instead of a per-segment dump. The exporter
path is already proven twice (`flags` and the four session keys: build the `w8v5` tree, install, delete
`Mods/Cache/CelesteTAS*`, re-run, and **assert the new key**).

### `movementCounter` IS in the trace - and with it the in-frame speed tool becomes exact

My round-114 note ("the trace cannot carry the counter, so the exporter has to") was **wrong**: the
exporter walks the player's base chain, and `p.movementCounter` is already there next to
`p.dashTrailCounter` (row 299 has 126 player keys, and `movementCounter = [0,0]`). No exporter change was
needed; only the analysis was wrong.

Switching the tool to `move = dpos + dcounter` (and dropping the 5 px/s buckets for 0.5 px/s) turns it
from a noise detector into a signal extractor: the flag rate falls from 46% (9516/20507 frames) to **8.2%**
(1679/20507) over the 40 largest `mismatch` segments, and the deltas cluster on real constants:

```
implied - previous row's Speed:
  -169.5 x329   60 x165   -20 x133   -280 x65   -60 x45   -65 x44
  -209.5 x44   -21.5 x42   20 x40     25 x35    280 x33   21.5 x30
state pairs:
  StNormal->StNormal x573   StDash->StDash x392   StDummy->StDummy x274
  StSummitLaunch->StSummitLaunch x122   StLaunch->StLaunch x84   ...
```

Each of those numbers is a `Speed` write the simulator lands on a different frame than the game
(`-170` and `-210` are jump/wall family values, `60`/`20`/`280` smaller ones). The next step is to map
them to source writes - grep `Player.cs` for those constants in speed contexts, and for each one check
which frame the simulator performs it on - instead of hunting a single segment at a time. The tool is
`.tmp/recon3/in-frame-deltas.mjs` (report + trace, no build).

### The `-169.5` cluster is `SuperWallJumpH = 170f` - the value is right, the frame is not

Mapping the largest cluster of the corrected in-frame-speed tool (329 frames over the 40 largest
`mismatch` segments) names the mechanism immediately:

- source: `SuperWallJumpH = 170f` (`Player.cs:207`), written as `Speed.X = 170f * (float)dir;`
  (`Player.cs:2619`, inside `SuperWallJump`);
- simulator: `sim.rs:7843` - `p.speed = Vec2::new(170.0 * dir as f32, -160.0);` - with unit tests
  asserting `(-170.0, -160.0)`.

So the simulator already implements the *value*, which means the cluster is not a missing writer but a
**frame-placement** difference: one side applies the super wall jump's horizontal speed on a frame where
the other has not yet (or has already). That is exactly the shape the recon described, now attached to a
named code path (`dash_jump`'s super-wall-jump branch, whose trigger is the
`SuperWallJumpAngleCheck`-gated case at the top of `DashUpdate`).

Next: take three or four of the 329 frames, dump them through the gate (the dump prints `gameSpeed` and
`rustSpeed` per frame) and read which frame each side performs the write on - the branch conditions
(`SuperWallJumpAngleCheck`, the buffered-jump window, the facing) are all in the simulator already, so the
difference should be a single condition or a single frame of ordering.

### The `-170` cluster and a second, different frame: what the tail dumps actually show

Two things learned from dumping `7-Summit|0|g-01|134447` (833 frames) and `7-Summit|0|g-01|356101`:

1. **`--dump-segment` prints from the segment's first replayed frame, not the tail** - the divergence is at
   the end, so `-Last 4` is what to read. (Earlier rounds read the first lines of very short segments and
   were correct by accident.)
2. The g-01 divergence frame (row 135320, offset 832) is **not** the super-wall-jump write: both sides are
   `StNormal`, `gameMove = (-2.30556, +0.12500)` with `gameSpeed = (-138.33328, +15.00003)` against the
   simulator's `rustMove = (-2.30556, 0.00000)` with `rustSpeed = (-138.33328, +7.50001)`. So on that frame
   the game applied a second `+7.5` of vertical speed (0 -> 7.5 -> 15, i.e. gravity 900/60 = 15 in two
   half steps) and moved `+0.125 = 7.5 * dt` in y, while the simulator applied only one `+7.5` and moved 0
   in y. The counter moved identically on both sides (`+0.125`), so the sim's absent y move came from a
   `MoveV` it performed with a different speed (or not at all), not from the counter.
3. Also worth knowing for reading dumps: `rustMove` is the simulator's own recorded move total while
   `gameMove` is `dpos + dcounter`; they are the same quantity only when the counter moves the same way on
   both sides - which is why c-00's earlier reading looked contradictory.

Candidate readings for the y difference, both one-frame placements: gravity/variable-jump-height applied
one frame later in the simulator (the game's move uses the freshly stepped speed, the simulator's uses the
previous frame's), or the half-gravity branch (`Speed.Y < 0 && jump held`) taken one frame longer. The
frame is reproducible with the single command above; the next step is to print the simulator's
`speed.y`/`varJumpTimer`/jump-held state on the two frames around offset 831-832 and compare with the
trace's own `p.*` values.

### g-01: the game steps vy by +7.5 every frame; the simulator skips one step at the dash-end frame

The game's own fields (trace rows 135314-135322, all `jumpHeld=false`, `varJumpTimer` ~0, `onGround=false`)
show a clean sequence after the dash ends at row 135318:

| row | state | vy |
| --- | --- | ---: |
| 135318 | StNormal (dash just ended) | 0 |
| 135319 | StNormal | 7.5 |
| 135320 | StNormal | 15.0 |
| 135321 | StNormal | 22.5 |
| 135322 | StNormal | 30.0 |

So `+7.5` per frame is the ordinary gravity step in this context - not the jump-held half-gravity (the jump
is not held and `varJumpTimer` is ~0). The simulator matches the first step (0 -> 7.5 on 135319) and then
applies **0** on 135320 (its dump shows `rustSpeed.y = 7.50001` where the game has `15.00003`, and its y
move is 0 where the game's is `+0.125 = 7.5 * dt`). It therefore **skips one gravity step**, and that is
the divergence.

The likely site is the simulator's entry into `StNormal` from the dash: the game reaches `StNormal` on row
135318 and starts accelerating on the next row, so a one-frame suppression (a state-entry guard, a
`was_on_ground`/`was_normal` branch, or a `varJump` reset) in `normal_update` is the shape to look for. The
next check is a print of the simulator's vy and the gravity branch across rows 135318-135321, compared
against the table above - and the same signature should appear in the other `StDash -> StNormal`
transitions the classification counts (231 frames in the sampled 40 segments).

### Confirmed: the simulator gates the 1.2x grounded-ultra on `on_ground`, the source does not

Recon lead (class-3 `LostLevels|0|f-00`) checked against the current tree - note the recon's line numbers
(`sim.rs:8969`, `:6644`) are from an older revision and now point at unrelated code, so this was found by
meaning rather than by line.

- **Source**: `Player.cs:3318` - `if (DashDir.X != 0f && DashDir.Y > 0f && Speed.Y > 0f)` sets
  `Speed.Y = 0f; Speed.X *= 1.2f; Ducking = true;` (`:3320-3325`), i.e. the grounded-ultra launch. It sits
  inside the V-collision/landing block: outer guard `if (Speed.Y > 0f)` at `:3282`, preceded by the
  `OnDashCollide` dispatch (`:3275-3280`) and the corner-snap loops (`:3286-3309`). **There is no
  `onGround` condition.**
- **Simulator**: `sim.rs:7718-7722` requires `p.on_ground && p.dash_dir.x != 0.0 && p.dash_dir.y > 0.0 &&
  p.speed.y > 0.0 && !dream_block_below` - the `on_ground` term is an extra condition the source does not
  have. (A second site, `sim.rs:10043-10046`, uses the source's exact triple without `on_ground`, so the
  tree is inconsistent with itself as well.)

So the recon's suspicion was right: the simulator misses the 1.2x conversion whenever the dash's downward
collision happens on a frame where the sim's `on_ground` probe is false. The fix shape is to replace the
`on_ground` proxy with the signal the source actually has - "this dash frame collided downward" - which
means looking at how `dash_update`/the collision pass records a vertical block on the frame (the
`DreamDashCheck` and corner-snap code around `:7716` is already in the neighbouring lines, so the same
context is available). The `dream_block_below` term stays: it mirrors the `DreamDashCheck` arm at `:3311`.

### The remaining delta clusters map to named source constants

Mapping the corrected in-frame-speed clusters (40 largest `mismatch` segments) to `Player.cs`:

| cluster | source constant | where |
| --- | --- | --- |
| `-169.5` (~329 frames) | `SuperWallJumpH = 170f` | `:207`, written `Speed.X = 170f * dir` at `:2619` |
| `±280` | **`LaunchSpeed = 280f`** | `:289`, written `Speed = 280f * vector` at `:4941` |
| `60` | `HiccupAirBoost = -60f` / `SwimUnderwaterMax = 60f` | `:321` / `:705` |
| `-20` | `WallSlideStartMax = 20f` | `:185` |
| `-160` | `SuperWallJumpSpeed` / `MaxFall` / `EndDashSpeed` / `WallBoosterSpeed` | `:201` / `:125` / `:211` / `:677` |
| `130` | `WallJumpHSpeed` / `LiftYCap` | `:183` / `:293` |

The `280` cluster cross-checks with the state-pair histogram the same tool prints - `StLaunch->StLaunch`
(84 frames) and `StSummitLaunch->StSummitLaunch` (122) - so two independent views name the same mechanism.
The simulator implements all of these values (e.g. `SuperWallJumpH` at `sim.rs:7843`), which is why the
clusters are *frame placements*, not missing writes: the value is written, on a different frame than the
game writes it. The next cluster worth taking after the three workstreams in flight is `LaunchSpeed`.

### Next queued lead: `LaunchSpeed = 280` (both sides located)

The `+-280` cluster (state pairs `StLaunch->StLaunch` 84 frames, `StSummitLaunch->StSummitLaunch` 122)
corresponds to the launch write:

- source `Player.cs:4941` - `Speed = 280f * vector;` - followed by `Speed.Y <= 50f` -> `Speed.Y =
  Math.Min(-150f, Speed.Y); AutoJump = true;` (`:4942-4946`), the `explodeLaunchBoostTimer` reset
  (`:4947-4951`), and the direction snapping above it (`:4926-4940`);
- simulator `sim.rs:10764` - `p.speed = scale(direction, 280.0);` - with a unit test asserting
  `Vec2::new(-280.0, -150.0)` (`:14508`), so both the speed and the clamp are implemented.

As with the `SuperWallJumpH` cluster, the value is present on both sides, so this is a frame placement:
one side writes `Speed` on a frame where the other has not (or has already). The surrounding block is the
place to look - the `vector` snapping, the `-150` clamp with `AutoJump`, and the `explodeLaunchBoostTimer`
reset are all candidates for a one-frame difference, and each has a test to compare against.

### 926 of the 949 remaining mismatches diverge on the segment's LAST frame

Quantifying the current report (`gate-fl2-202.json`) by the gap between `frames` and `exactPrefixFrames`:

```
segments 1468   ok 518   mismatch 949
mismatch with exact == frames - 1 : 926 / 949      (97.6%)
ok       with exact == frames - 1 :   0 / 518
remaining mismatches              :  23            (21 with gap 2, 2 with gap 0)
```

Every one of the twelve largest mismatches is in that 926 (`5-MirrorTemple|0|void` 855/854,
`7-Summit|0|b-09` 848/847, `7-Summit|0|g-01` 833/832, ...), and no `ok` segment has the shape. So the
remaining gap is **not** 949 independent mechanisms: 97.6% of it is a single systematic difference on the
last frame of a segment.

Two candidate explanations, both cheap to check in the harness rather than in the physics:

1. **The last compared row belongs to the next run.** Segments are maximal runs of `(sid, mode, room)`; if
   the window's final row is the first row of the following run (a different room, or the transition
   frame), then the comparison at that row is between the simulator's state in *this* room and the game's
   state in the *next* one - a guaranteed mismatch that says nothing about fidelity. The harness's
   `Segment` struct (`examples/tas_fidelity.rs:783`, `start_row` at `:789`, `frames` at `:795`) and the
   loop that fills it decide this.
2. **The last frame is the room transition itself**, which the harness deliberately does not replay (the
   anchor/restore model works per segment). In that case the count is honest but the *metric* is
   pessimistic, and the real per-frame fidelity is much higher than 518/1468 suggests.

Either way this reframes the objective: before hunting more one-frame mechanisms, read the segment-filling
loop and settle which of the two it is. If it is (1), excluding the boundary row is a harness fix that
would reclassify the great majority of the 926 in one step - the single highest-leverage change left.

### RETRACTION: `exact == frames - 1` is a tautology, not a boundary artefact

The previous note claimed that 926 of 949 mismatches sharing `exactPrefixFrames == frames - 1` was a
systematic last-frame boundary artefact. **That is wrong, and the statistic carries no information**: the
harness replays a segment until the first diverging frame and stops, so the divergence is *by construction*
on the last frame it replayed (`exact == frames - 1`), while an `ok` segment - having no divergence - ends
with `exact == frames`. The `0/518` for `ok` segments is the same fact seen from the other side.

The only real signal in that table is the **23 exceptions**: 21 segments with a gap of 2 and 2 with a gap of
0. A gap of 2 means the harness did not stop at the first frame whose fields differed (a stalled or
non-mutating row is involved), and a gap of 0 means a segment with no exact prefix at all. Those 21+2 are
what to look at, not the 926.

The metric that does carry information is how many frames each segment replays before diverging - i.e. the
`frames` column and the class analysis built on it - which is how the mechanisms in the notes above were
found in the first place.

### The skipped gravity step is a ground probe that disagrees with the game

Found the site. `normal_update` applies gravity only when airborne:

```rust
if !p.on_ground {                                            // sim.rs:7462
    p.speed.y = approach(p.speed.y, fall_target, GRAVITY * gravity_mult * p.frame_delta_time);
}
```

with `gravity_mult` halved by the earlier branch at `:7454-7455`, which is why the observed step is
`900 * 0.5 / 60 = 7.5`.

The g-01 frame (row 135320) is therefore explained by the simulator believing it is **on the ground**: both
sides have `speed.y = 7.5` entering the frame (so the `Speed.Y >= 0` half of the probe agrees), and
`p.player_on_ground` is computed at `:6826` as `speed.y >= 0.0 && grounded(p, map)`. The only remaining term
is `grounded(p, map)` - so the simulator found a solid beneath the player where the game found none, which
is why it skipped the step and why its `vy` stays at 7.5 while the game's goes to 15.

That points the search away from gravity and at the **map/geometry**: print `grounded(p, map)` and which
rectangle `solid_at` returns on rows 135318-135321. It is worth checking the solid kinds added in the last
few rounds (`TempleGate`, `FloatySpaceBlock`, `SwitchGate`/`TouchSwitch` dash switches) as the possible
phantom: all of them measured zero per-segment regressions, which is a statement about segments that
already diverged, not about collisions they might newly introduce. The same signature should be checked on
the other segments in the `StDash -> StNormal` family before fixing anything.

### The skipped gravity step is not the ground probe: `speed.y` is reset instead

Instrumented the probe at `sim.rs:6826` (printing `grounded(p, map)`, the pre-gravity `speed.y`, and every
entity whose bounds intersect the probe rectangle) and ran the `g-01` slice. Result:

```
GROUND pos=(26076,-19084) vy=0.000 geo=false entities=[]
GROUND pos=(26077,-19082) vy=0.000 geo=false entities=[]
GROUND pos=(26081,-19082) vy=0.000 geo=false entities=[]
GROUND pos=(26082,-19082) vy=0.000 geo=false entities=[]
```

Two things follow, and both change the search:

1. **No phantom solid.** `geo` is `false` on every probed frame near the divergence and no entity overlaps
   the probe rectangle, so the previous note's hypothesis (a newly added solid kind making the simulator
   think it is grounded) is **refuted**. The `TempleGate`/`FloatySpaceBlock`/`SwitchGate` additions are not
   implicated here.
2. **The simulator's pre-gravity `speed.y` is 0** on those frames while the game's is 7.5 then 15. The
   gravity gate `if !p.on_ground` (`:7462`) is therefore satisfied and gravity *is* applied - the value
   being stepped is simply the wrong one, because something reset `speed.y` to 0 earlier in the frame. The
   bug is a **`speed.y` reset**, not a ground probe: candidates are the dash-end path (`DashEnd`'s
   `Speed.Y` handling), the publish block, or a `var_jump`/`varJump` write.

Next: print `speed.y` at three points in the frame (before the state callback, after it, after the publish)
on rows 135318-135321 and find which write takes it to 0, then compare that site with
`vendor/celeste-fna/Celeste/Player.cs`'s `DashEnd`/`NormalBegin`.

### Bracketed: on the second `StNormal` frame the callback adds no gravity at all

Two probes (before the state callback at the wind call, after it at `tick_lift_speed`) on the `g-01` slice:

```
VY pre  pos=(26084,-19082) vy=0.0000 state=Normal
VY post pos=(26084,-19082) vy=7.5000 state=Normal     <- first StNormal frame: gravity applied
VY pre  pos=(26082,-19082) vy=7.5000 state=Normal
VY post pos=(26082,-19082) vy=7.5000 state=Normal     <- second frame: nothing added
```

So the gravity block is **not executed** on the second `StNormal` frame - nothing writes and then resets it;
the callback simply does not apply it. The gate is `if !p.on_ground` (`sim.rs:7462`), so `p.on_ground` must
be true at that moment in the simulator while the trace says the game's player is airborne with
`speed.y = 7.5 -> 15`.

**Caveat on the previous note**: `pos` is not a unique key across frames, and the ground probe's
`geo=false` lines may well have come from *other* frames that share the same coordinates - so "no phantom
solid" should be treated as unproven until the probe prints `p.on_ground` as well and is matched by row
rather than by position.

Next probe (cheap): print `p.on_ground`, `p.player_on_ground`, `grounded(p, map)` and `p.speed.y` in the
same line, and match by the *row* the gate reports (the dump gives the row index) instead of by position.

### New suspect: the simulator's `jump_held` is false where the trace holds jump

The same-line probe (`p.on_ground`, `p.player_on_ground`, `grounded(p, map)`, `p.speed.y`, state and the
frame's inputs) on the `g-01` slice:

```
OG pos=(26084,-19082) vy=0.0000 state=Normal geo=false pog=false og=false in=(1,0,false,false)
OG pos=(26082,-19082) vy=0.0000 state=Normal geo=false pog=false og=false in=(1,0,false,false)
```

Two readings:

1. `geo`, `player_on_ground` and `on_ground` are **all false** on the probed frames, so the "phantom solid"
   suspicion does not hold on this path (this time the four values come from the same line, which is
   stronger evidence than the earlier position-matched log).
2. **`jumpHeld=false`** - while the trace says `in.jump = true` on rows 135319-135323 (`mx=1`, `my=0`, which
   do match). The simulator is not seeing the held jump on those frames, and `jump_held` is exactly the
   input the variable-jump half-gravity branch reads (`sim.rs:7454-7455` for the multiplier, `:7469-7475`
   for `var_jump_timer`), which is the context the missing gravity step lives in.

Caveat kept from the previous note: `pos` (and even `pos + inputs`) is not a guaranteed unique key across a
frame, so this needs to be confirmed by matching on something that carries a row index - e.g. printing a
per-run frame counter alongside, or dumping the segment with the gate and comparing the row the gate names.

If it confirms, the search moves to the harness's input plumbing for `jump_held` (`tas_fidelity.rs`'s
`InputRec` -> `InputState`), not to `normal_update`.

### RETRACTION: `jump_held` is a straight copy of `in.jump` - the probe was misaligned again

Checked the plumbing instead of chasing the probe: `tas_fidelity.rs:119` is `jump_held: self.jump`, and
`InputRec.jump` is the trace's `in.jump` (`#[serde]` field `jump` in the struct at `:85`). So
`input.jump_held` *is* the trace's held-jump flag - there is no input-pipeline bug, and the probe's
`jumpHeld=false` on a frame whose trace row says `in.jump = true` was **another row-misalignment artefact**,
not a finding. The "new suspect" in the previous note is withdrawn.

This is the third time position-based matching has produced a wrong conclusion in this family (phantom
solid, speed.y bracket, jump_held). The rule to keep: **a probe must print something that identifies the
row uniquely** - the trace exports `sceneTimeActive` per player row, and the simulator's `p.scene_time_active`
advances once per active frame, so printing both sides' value is a real key. Position, inputs, or a
combination are not.

What still stands from the last two notes is the *bracketed* fact, which did not depend on position
matching: entering `StNormal` the callback applies gravity on the first frame (0 -> 7.5) and adds nothing on
the second (7.5 -> 7.5) where the game goes to 15. That remains the thing to explain, and the next probe
should carry `scene_time_active` so its frames can be tied to the gate's row numbers.

### With the unique key in place: gravity steps normally, and the earlier "skipped step" is unconfirmed

Re-ran the probe carrying `p.scene_time_active` (the trace exports `sceneTimeActive` per player row, and the
simulator advances `p.scene_time_active` once per active frame, so the two are a real key - unlike position).
Filtering the log for `state=Normal` frames near the target:

```
t=8.000052 vy=7.5000 geo=false og=false
t=8.016719 vy=15.0000 geo=false og=false     <- next frame: gravity applied
t=4.450006 vy=7.5003 geo=false og=false
t=4.466672 vy=15.0003 geo=false og=false     <- applied again
t=5.833357 vy=7.5001 geo=true  og=true       <- grounded frame, skipping gravity is correct
```

So the simulator does step `7.5 -> 15` on ordinary airborne frames, and the one frame that skips it has
`og = true` legitimately. That makes the round-129 bracket ("the callback adds nothing on the second frame")
**unconfirmed**: it was most likely taken on a grounded frame, which is correct behaviour, not a bug. The
earlier note stays withdrawn until the probe line for the *specific* diverging frame is identified.

To finish this: the diverging frame's `scene_time_active` (from the trace) has to be matched against the
probe log. The inline-node attempt to read it failed on quote escaping - use a script file for that, as the
other readers do, and match the key rather than the position. Until then, treat the "gravity step" family as
**open**, not diagnosed.

### UNIQUELY IDENTIFIED: `speed.y` is zeroed between the two `StNormal` frames

The probe (at the ground check, i.e. before the state callback) prints both position and `vy`, and the
frames are unique by `pos + state + vy`; the run is contiguous, so the sequence is unambiguous:

```
ST t=12.799979 pos=(26087,-19082) vy=0.0000 state=Dash
ST t=12.816646 pos=(26084,-19082) vy=0.0000 state=Normal    <- first StNormal frame
ST t=12.833312 pos=(26082,-19082) vy=0.0000 state=Normal    <- second StNormal frame, still 0
```

Combined with the gate dump's end-of-frame speeds (7.5 at the end of the first frame, 7.5 again at the end
of the second, where the game has 15):

| frame | vy at probe (frame start) | vy at frame end |
| --- | ---: | ---: |
| first `StNormal` | 0 | **7.5** (gravity applied) |
| second `StNormal` | **0** (game has 7.5) | 7.5 |

So gravity is not skipped at all: the value it steps is 0 because **the 7.5 written at the end of the first
`StNormal` frame is gone before the second frame's ground check**. One write zeroes `speed.y` between the
two frames - either late in the first frame's tail or early in the second, before `sim.rs:6826`.

That also explains every earlier observation coherently: frame-start `vy = 0` -> y move 0 (the dump's
`rustMove.y = 0`), and frame-end `0 + 7.5 = 7.5` instead of the game's `7.5 + 7.5 = 15`.

Candidate sites, in the order worth checking: the `var_jump_timer` block (`sim.rs:7469-7475`, which runs
right after the gravity step), a `DashEnd`/`NormalBegin` transition write, `tick_lift_speed`, and anything in
the frame tail that treats a just-ended dash specially (`dash_attack_timer`/`end_dash`). The next probe
should print `speed.y` at the *end* of `step` (or at the top of the next frame before the timer block) so the
write is bracketed to one of those.

### BRACKETED to `step`'s early input/timer region: `speed.y` 7.5 -> 0 between two frames

The end-of-`step` probe (anchored at the five `p.on_ground = grounded(p, map);` tails) prints

```
TAIL pos=(26084,-19082) vy=0.0000 state=Normal varJump=0.0000 dashAtk=0.1167   <- row 135318
TAIL pos=(26082,-19082) vy=7.5000 state=Normal varJump=0.0000 dashAtk=0.1000   <- row 135319
```

and the frame-start probe (at the ground check, `sim.rs:6826`) for the *next* frame printed `vy=0.0000` on
`pos=(26082,-19082)`. Both probes also match the gate dump's own numbers (offset 830 = 0, offset 831 =
7.5), which validates the row-to-frame mapping used through this investigation.

So the sequence is unambiguous:

| frame ends at row | vy at frame start | vy at frame end |
| --- | ---: | ---: |
| 135318 | - | 0 |
| 135319 | 0 | 7.5 |
| 135320 | **0** (previous frame ended at 7.5) | 7.5 (game: 15) |

The write that zeroes `speed.y` therefore happens **after the end of one frame and before the ground check
of the next** - i.e. in the early part of `step`, in the input/timer region between `sim.rs:6637` and
`:6826`, and *not* in `normal_update` and not in the physics tail. `var_jump_timer` and `dash_attack_timer`
are both 0 / still counting at those frames, so neither block is obviously the culprit; the next probe
should print `speed.y` at a few points inside that region (after the input buffers, after the force-move
handling, after the wind controller) to name the write.

### Ordered HEAD/TAIL log: the 7.5 is lost between the frame tail and the next frame's ground check

Both probes in **one** binary and one run (so the ordering is exact - the previous note compared two
separate runs, which left the gap imprecise):

```
TAIL pos=(26084,-19082) vy=0.0000 state=Normal vj=0.0000 da=0.1167   <- end of row 135318's frame
HEAD pos=(26084,-19082) vy=0.0000 state=Normal vj=0.0000 da=0.1000   <- start of the next frame
TAIL pos=(26082,-19082) vy=7.5000 state=Normal vj=0.0000 da=0.1000   <- end: gravity applied, matches the game
HEAD pos=(26082,-19082) vy=0.0000 state=Normal vj=0.0000 da=0.0833   <- start of the next frame: 0, not 7.5
TAIL pos=(26080,-19082) vy=7.5000 state=Normal vj=0.0000 da=0.0833   <- end: 7.5 where the game has 15
```

The `TAIL` probe sits on the last statement of each `step` branch (`p.on_ground = grounded(p, map);`
followed by `Ok(())`), so the loss is strictly inside `step`'s opening region, `sim.rs:6637-6826`, before
the ground check. Within that region the only direct write to `p.speed` is the death/respawn path at
`:6699` (`if p.dead { ... p.speed = Vec2::default(); ... }`), which does not apply here - so the value is
lost **through a called function**, most plausibly an `end_dash`-style restore that writes `speed` from a
snapshot (`before_dash_speed`) or a state-entry helper.

Also noted, independently: `dash_attack_timer` decrements only on every second frame in this window
(0.1167 -> 0.1000 -> 0.1000 -> 0.0833), which is odd in `StNormal` where the source only decrements it
inside `DashUpdate`; that may be a second, separate frame-placement difference worth checking later.

Next probe: print `p.speed.y` just before and just after the calls in that region (the input-buffer block
ends at `:6691`, and the region ends at the ground check) so the losing call is named in one iteration.

### NAMED: `apply_wind_movement` zeroes `speed.y` through the `windMovedUp` / `maxFall = 0` chain

Three probes inside `step`'s opening region, in one binary and one run, ordered:

```
CHK after_moving_solids pos=(26082,-19082) vy=7.5000 state=Normal    <- still 7.5
CHK after_wind          pos=(26082,-19082) vy=0.0000 state=Normal    <- apply_wind_movement zeroed it
CHK after_tick_timers   pos=(26082,-19082) vy=0.0000 state=Normal    <- unchanged after
```

`tick_timers` is a pure timer decrement loop (`sim.rs:7123-7152`), so it was correctly ruled out. The
position is **identical** across the first two probes, i.e. the wind move went into the movement counter
(sub-pixel, no collision) - so this is not a blocked move calling an on-collide that zeroes the speed.

The mechanism is the `windMovedUp` chain. The room's wind is an updraft (`wind = (0,-400)`), so
`apply_wind_movement` sets `windMovedUp = true` (source: `Player.cs:3128-3131`). In `Player.NormalUpdate`
the fall target becomes `0` while `windMovedUp` holds (the branch I read earlier at `:3725`:
`windMovedUp && ... ? -32 : ... (!windMovedUp ? 40 : 0)`), so `approach(Speed.Y, 0, Gravity * dt * mult)`
pulls `7.5` to exactly `0` in one step - which is what the probe shows, and it matches the observed step
size `GRAVITY * 0.5 * dt = 7.5` exactly.

The game's frame keeps `maxFall = 160` (its `vy` goes 7.5 -> 15), so the simulator takes a `windMovedUp`
branch the game does not. That is now a named, testable defect: compare the simulator's condition for
`windMovedUp` (and where it feeds `max_fall`) against `Player.cs:3116-3131` and `:3725`.

### Complete chain: the updraft's 1-px step is blocked ABOVE, and the collide handler zeroes speed.y

`apply_wind_movement`'s y branch (`sim.rs:11438-11448`) does exactly one thing:

```rust
let mut move_y = p.wind.y * WIND_MOVE_MULT * p.frame_delta_time;    // -400 * 0.1 * dt = -0.667 (up)
if move_y != 0.0 && (p.speed.y < 0.0 || !grounded(p, map)) {
    move_axis_amount(p, map, false, move_y);
}
```

and `move_axis_amount` (`:9855`) adds the amount to the counter, rounds it (`round_ties_even(-0.667) = -1`,
so it really attempts a 1-pixel step **up**), and on a blocked step clears the counter and runs the collide
handler, which in `StNormal` zeroes `Speed.Y` - the standard ceiling response.

The CHK probes measured **no position change** across `apply_wind_movement`, which is only possible if that
1-px upward step was **blocked**. So the mechanism is: a solid sits 1 px above the player in the simulator's
world, the updraft tries to move into it, and the ceiling response kills the fall speed (7.5 -> 0 in one
frame), whereas the game keeps falling (7.5 -> 15).

**Correction to an earlier round.** The round-128 probe concluded "no phantom solid" - but it probed
*downwards* only (`grounded(p, map)` and `current_player_rect(p, pos.x, pos.y + 1.0)`). The blocker here is
*above*, so that probe could not have seen it, and the conclusion should have been "no phantom solid below".
That directional blind spot is worth remembering: a ground probe says nothing about ceilings.

Next: print `map.non_dream_solid_at` for `pos.y - 1` (and which entity/kind overlaps that rect) on the same
frames, and compare with the trace - if the sim has a solid above where the game does not, the culprit is
one of the solid kinds (the recently added `TempleGate`/`FloatySpaceBlock`/`SwitchGate` are candidates given
`7-Summit`), and the fix is in that kind's geometry rather than in the wind.

### The blocker above is a `solids` rectangle, not an entity - and the game must not have it

Probing **upwards** this time (`current_player_rect(p, pos.x, pos.y - 1.0)`, `map.non_dream_solid_at`, and
every entity overlapping that rect):

```
UP pos=(26084,-19082) vy=0.0000 windy=-400.0 solidAbove=true ents=[]
UP pos=(26082,-19082) vy=7.5000 windy=-400.0 solidAbove=true ents=[]
```

`solidAbove` is **true** while `ents` is **empty**, so the blocking geometry is a rectangle in
`map.solids` - the level's static solid list - and **not** one of the entity kinds added in recent rounds
(`TempleGate`, `FloatySpaceBlock`, `SwitchGate`, dash switches). That hypothesis is refuted.

This creates a sharp contradiction, and it is the useful part:

- the gate dump shows `gamePos == rustPos` on those frames, so both sides are at the same coordinates;
- if the game's world had the same solid above, the game's own `MoveV(-0.667)` in `Player.WindMove` would
  collide on its first whole-pixel step and `Player.OnCollideV` would zero `Speed.Y`;
- yet the game's `vy` rises 7.5 -> 15 on exactly that frame.

So **the game's world does not have that solid**, or its wind's upward move does not reach it. The next
probe should name the offending rectangle (walk `map.solids`, print the one that intersects the probe rect
with its coordinates) and compare it against the level data for `7-Summit` room `g-01` - a rect that the
simulator carries and the game does not is a *map decode* difference, which would be a much broader defect
than anything in the wind code.

### The tile above is real; the divergence moves to the DOWNWARD probe (`OnGround`)

Naming the rectangle:

```
RECT pos=(26084.0,-19082.0) up=(26080.0,-19089.0,8.0,6.0) solids=[rect(26080,-19096,8,8)]
RECT pos=(26082.0,-19082.0) up=(26078.0,-19089.0,8.0,6.0) solids=[rect(26080,-19096,8,8)]
```

The solid is an honest 8x8 level tile at `(26080,-19096)`, its bottom at `-19088`; the probe rect shows the
player is in the **ducking** collider (8x6, top at `-19089`), so they overlap by exactly one pixel. This is
**not** a map-decode difference and not an entity: the tile is legitimately there, and both sides are at the
same coordinates.

What follows is sharper. If the game had performed that upward wind move, its ducking collider would overlap
the same tile by the same pixel, `Player.OnCollideV` would zero `Speed.Y`, and its `vy` could not rise - but
it does rise (7.5 -> 15). So the game **never executes the upward wind move** on that frame. The only clause
in `Player.WindMove`'s y guard that can stop it while `Speed.Y = 7.5 > 0` is `OnGround()`:

```
if (!(base.Bottom > (float)level.Bounds.Top) || (!(Speed.Y < 0f) && OnGround())) return;   // Player.cs:3116
```

The simulator's equivalent, `p.speed.y < 0.0 || !grounded(p, map)`, passes because its `grounded` is
**false** (measured in round 128's probe) on the very same frame. So the disagreement is in the **downward
probe**: the game considers the player grounded, the simulator does not. Celeste's `Player.OnGround()`
overrides `Actor.OnGround()` and does not simply test the full collider, so the probe rectangle is the prime
suspect - and that is a read of `Player.OnGround`/`Actor.OnGround` in the vendored source, not another
instrumented run.

### Correction: the probes are equivalent, so `OnGround` is an unlikely culprit - `noWindTimer` is the better one

Compared the two implementations:

| | implementation |
| --- | --- |
| source `Actor.OnGround(downCheck = 1)` (`Actor.cs:134`) | `CollideCheck<Solid>(Position + UnitY * downCheck)`, else `CollideCheckOutside<JumpThru>(Position + UnitY * downCheck)` |
| simulator `grounded_at_offset(p, map, 1.0)` | `map.solid_at(rect at +1)` or `map.jump_thru_at(that rect, current collider bottom)` |

`Player` does **not** override `OnGround` (the earlier note assumed it did - there is no such method in
`Player.cs`), and the simulator's version mirrors the source's structure: the same +1 offset, the same
Solid-then-JumpThru order, and the JumpThru case passing the current bottom, which is what
`CollideCheckOutside` means. So `OnGround` is a weak candidate for the disagreement, and round 139's
inference ("the game considers the player grounded") should be treated as unproven.

The better candidate is the guard's **first** clause, which the previous note skipped:

```csharp
if (JustRespawned || !(noWindTimer <= 0f) || !InControl
    || StateMachine.State == 4 || StateMachine.State == 2 || StateMachine.State == 10) return;   // Player.cs:3085
```

`noWindTimer` is set by the wall-jump family, so a frame shortly after a wall jump - which is exactly the
context here (the player has just left a dash with `wallSpeedRetained` live) - is precisely where the game
would return and the simulator would push the wind. The next probe should print `p.no_wind_timer` alongside
the wind push on those frames; that is a one-line check and it discriminates between the two clauses
directly.

### `noWindTimer` is 0 on BOTH sides - refuted in one measurement; only `Bounds.Top` is left

The one-line discriminator, printed on the same frames:

```
sim : pos=(26082,-19082) vy=7.5000 windy=-400.0 noWind=0.0000 state=Normal
game: row=135320     state=StNormal noWind=0 vx=-138.333 vy=15.000 pos=(26080,-19082)
```

So `noWindTimer` is not the clause that stops the game's wind push. Walking `Player.WindMove`'s guards with
the measured state (`StNormal`, `InControl`, not respawning):

1. `JustRespawned` - no;
2. `!(noWindTimer <= 0f)` - no, both are exactly 0;
3. `!InControl` - no;
4. `State == 4 || 2 || 10` - no, the state is `StNormal`;
5. `move.X != 0f && State != 1` - the wind is `(0,-400)`, so the x half does not run;
6. **y half**: `if (move.Y == 0f) return;` - no;
7. **`if (!(base.Bottom > (float)level.Bounds.Top) || (!(Speed.Y < 0f) && OnGround())) return;`**

Clause 7 is what remains. Its second half needs `OnGround()` true, and the simulator's equivalent probe is
structurally identical to `Actor.OnGround` (previous note), so the first half is the better candidate: the
player's bottom being at or above the **room's top bound** would make `!(Bottom > Bounds.Top)` true and the
game would return before pushing.

Next check, cheap and decisive: print the player's bottom (`pos.y + collider height`), the simulator's
`current_room_bounds.top()` and `map.bounds.top()` on those frames, and compare with the trace's own
position - if the simulator's room bounds disagree with the level's, the wind guard flips for reasons that
have nothing to do with wind.

### The 23 "gap != 1" mismatches are explained - and one of them is a named, reproducible case

Listing every `mismatch` whose `frames - exactPrefixFrames != 1`:

- **21 have `gap = 2`**, and every one of them carries a large `stalledFrames` count (36, 343, 48, 45, 57,
  45, 54, ...). A stalled frame does not compare fields (the player did not move), so the exact prefix skips
  it - which is exactly a gap of 2. So these are an artefact of the stalled-frame accounting, not a separate
  class of defect, and round 125's open question ("what are the 21 exceptions?") is closed.
- **`3-CelestialResort|0|roof07|42489`** is the real outlier: `frames=6`, `exactPrefixFrames=0`,
  `leading=1`, `stalled=57`, i.e. no exact frames at all. This is the case the round-45 recon already
  measured: `gameMove = +1.0` per frame against `rustMove = 0.0` with the player in a frozen state - the game
  keeps pushing the player one pixel per frame during a rooftop cutscene/freeze and the simulator does not.

That last one is a named, reproducible defect with its own signature (`+1.0/frame`, frozen player,
`roof07|42489`, 6 replay frames), and it is unrelated to the wind/gravity family pursued in the notes above.
It is the better next target of the two: it is small, it is measured, and the mechanism is presumably a
camera/bounds push that `Player.Update` performs even while `Engine.FreezeTimer` suppresses ordinary updates.

### Correction on `roof07`: not a frozen camera push - a blocked player creeping +1/frame

The trace rows for `3-CelestialResort|0|roof07|42489` (the segment with `exactPrefixFrames = 0`):

| rows | x | vx |
| --- | ---: | ---: |
| 42489-42494 | 8230 -> 8235 (**+1 per frame**) | 323.333 |
| 42495 onward | 8236 (frozen) | 323.000 |

So the player is not frozen by a cutscene: it is moving with `vx = 323.333`, which is `5.39 px/frame`, yet
the position advances by exactly **one pixel per frame** for six frames and then stops entirely. That is a
blocked-player-at-a-boundary shape, not a camera push, and it corrects the previous note (which read the
round-45 recon's "frozen" label literally).

The simulator advances **0** px on those frames, so it is the more conservative of the two. The +1 creep is
the interesting quantity: it is small enough to be a `Level.EnforceBounds` interaction (the clamp writing the
player back to the bound after the move, with the movement counter carrying one pixel through), or a
one-pixel `MoveHExact` corner correction. Room `roof07`'s bounds are the next thing to read - if the player
is at the right edge, the comparison is `x + width` against `Bounds.Right`, and the trace's `collider` field
gives the exact rectangle to compare.

### `roof07`: the player is outside the room's left bound and `EnforceBounds` clamps it at +1/frame

Room `roof07` in `3-CelestialResort` has bounds `x = 8232, width = 488`. The trace:

| row | collider | vx | pos.x |
| --- | --- | ---: | ---: |
| 42494 | `[8231,-796,8,11]` | 323.333 | 8235 |
| 42495 onward | `[8232,-796,8,11]` | 323.000 | 8236 |

So the player's collider left edge ends up **exactly on the room's left bound (8232)** and stays there, while
`vx` says `5.39 px/frame`. Rows 42489-42494 show it creeping right at exactly **+1 px/frame** from `x = 8230`,
i.e. it is entering the room from outside and `Level.EnforceBounds` is pulling `player.Left` back to
`Bounds.Left` every frame. The simulator advances 0 px there.

The concrete suspicion to check next is in the simulator's own clamp:

```rust
let bounds = p.current_room_bounds.unwrap_or(map.bounds);
```

If `p.current_room_bounds` is not set to this room's bounds during that window, the clamp falls back to
`map.bounds` - the bounds of the **whole map**, not the room - and the push is a completely different
quantity (or none at all). `Player.EnforceBounds` in the source always uses `this.Bounds` of the current
`Level`, i.e. the room. A one-line print of `p.current_room_bounds` and `map.bounds` on those frames decides
it, and if that is the cause the fix is in whatever is supposed to populate `current_room_bounds`, not in the
clamp itself.

### `current_room_bounds` is not the roof07 cause - and the unexplained quantity is a per-frame +1

Read rather than probed, which saved a round:

- `p.current_room_bounds` is written in exactly **one** place, `sim.rs:11340` (`p.current_room_bounds =
  next_room;`), i.e. during a room **transition**. It is not restored at a segment anchor.
- That looks dangerous given `let bounds = p.current_room_bounds.unwrap_or(map.bounds);` in the clamp - but
  the fallback is **correct**: the harness decodes the map **per room**, and `Map::bounds` is set from the
  room's own rectangle (`map.rs`: `bounds: level_room_bounds(x, y, width, height)`). So `map.bounds` *is* the
  room's bounds and the clamp uses the right rectangle.

So the geometry source is not the defect. What remains unexplained is the creep itself: `vx = 323.333`
(5.39 px/frame) producing exactly **+1 px per frame** for six frames, then zero. A one-off clamp
(`player.Left = Bounds.Left`) would be a multi-pixel jump on the first frame and then nothing, so the +1 has
to be something that **re-applies one pixel every frame** - the shape of a per-frame `MoveHExact(1)` corner
correction (`Player.cs:3288-3308` has such loops, though they are written for the dash states) or a
per-frame bounds push. The trace's `collider` field is the right instrument: `[8231,...]` at row 42494 and
`[8232,...]` from 42495 onward pins the transition to the frame where the left edge reaches the bound.

### The source's clamp is a hard set, so the roof07 +1 is not `EnforceBounds` - and `vx` is frozen

Read the left-bound branch (`Level.cs:2741-2748`):

```csharp
if (player.Top >= bounds.Top && player.Bottom < bounds.Bottom
    && Session.MapData.CanTransitionTo(this, player.Center - UnitX * 8f)) { ... NextLevel ...; return; }
player.Left = bounds.Left;      // hard set, no creep
player.OnBoundsH();
```

So the clamp teleports the left edge onto the bound; it cannot produce a +1-per-frame creep, and round 145's
attribution to `EnforceBounds` is withdrawn.

The sharper observation is in the `vx` column itself: across rows 42489-42495 `vx` is **constant** at
323.333 (then 323.000). A player actually running `NormalUpdate` would have its `vx` changed by friction or
acceleration every frame, so the player is **not** being updated normally during those frames while its
position still advances by exactly one pixel per frame - which is the shape of a **frozen/cutscene window**
in which something else nudges the player, and where the stored state name (`StNormal`) says nothing about
what is running. That also makes the segment's `leadingSkippedFrames=1` / `stalledFrames=57` less surprising:
this window is special, not ordinary gameplay.

So the next question is not about bounds at all: what advances a player by one pixel per frame while its
speed is held constant in `3-CelestialResort`'s rooftop room? Candidates are the rooftop cutscene's camera
pan with the player attached, a `DummyWalkTo`-style scripted walk, or a `Player.Update` early-out that still
runs one exact move.

### roof07: it behaves like a one-pixel-per-frame solid push, not a bounds clamp

Re-reading the same rows with the frozen-`vx` fact in hand:

- `vx` is **constant** at 323.333 across the six frames, which no `NormalUpdate` can produce (friction or
  acceleration writes `Speed` every frame).
- `pos.x` climbs **monotonically** 8230 -> 8236 and stops at exactly `room.Left + 4`, the +4 being the
  hitbox's left offset, so the stop condition is "the collider is fully inside the room".
- Neither `MoveHExact`-style moves nor a hard assignment (`player.Left = bounds.Left`, `Level.cs:2746`)
  touch `Speed`, so a constant `Speed` with a changing position is exactly their signature.

Both of those point at the **solid-push** family rather than at the bounds: an actor embedded in a solid is
nudged out one pixel per frame. That mechanism already has precedent in this project (the early `X3` work on
"solid push one pixel"). The left edge of `roof07` is where the room meets whatever is outside it, so a
player entering from the left can genuinely start inside a tile.

Next check: name the solid overlapping the player's collider on rows 42489-42494 (the same `map.solids` probe
used for the ceiling case, but with the player's own rect), and compare with the trace - if the game pushes
out of a tile that the simulator does not even report as overlapping, the defect is in the collider/position
bookkeeping, and if the simulator reports the overlap but does not push, it is in the push path.

### roof07: `enforce_level_bounds` is never called in those frames - the simulator takes an early return

The probe was placed unconditionally at the top of `enforce_level_bounds` (before its guard) and the
`--rooms roof07` run produced **zero** probe lines in 12 replayed frames (`simulated=2`, `frames=12`,
`exact=0`, `stalled=114`). So the function is never reached: on every one of those frames `step` takes an
early return - the freeze path (`sim.rs:6314-6317`) or the stall path - and the simulator performs no player
update at all, while the game nudges the player one pixel per frame.

That is consistent with everything measured before: constant `vx` (nothing writes `Speed`), a monotone
one-pixel creep (an exact move or a hard set, neither of which touches `Speed`), and a stop exactly when the
collider is fully inside the room. And it names where the fix belongs: **the early-return path**, not the
clamp and not the wind.

The distinction to settle next is freeze versus stall, because they mean different things:

- `Celeste.Freeze` skips `Scene.Update` entirely, so *nothing* should move - if the trace's rows in this
  window are frozen engine frames yet the player moves, the window is not a freeze;
- a **stall** (the room-transition shape the harness models: entities update, the Player does not) is the
  natural home for a one-pixel nudge, because solids, springs and camera code all still run.

So: read `step`'s freeze and stall early-return blocks (`sim.rs:6290-6320`) against the trace row's
`transitioning` / freeze fields for row 42489 onward, and find out which one this window is.

### roof07 is a TRANSITION (stall), not a freeze - `trans=true`, `freeze=0` on every row

The trace's own fields settle the freeze-versus-stall question:

```
row=42489 trans=true freeze=0 col=[8226,-796,8,11] vx=323.333 pos=8230
row=42494 trans=true freeze=0 col=[8231,-796,8,11] vx=323.333 pos=8235
row=42495 trans=true freeze=0 col=[8232,-796,8,11] vx=323.000 pos=8236
```

`transitioning` is true and `freezeTimer` is 0 throughout, so this is a room **transition**: `Player.Update`
is paused (which is exactly why `vx` stays constant at 323.333 - nothing writes `Speed`) while entities keep
updating, and one of them advances the player by one pixel per frame until its collider is fully inside the
room. The simulator takes the stall early return and does nothing.

That closes the classification and re-points the fix: it is neither the clamp (`enforce_level_bounds` is not
even called) nor the wind. Something in the **transition** moves the player one pixel per frame in the game.
Candidates, in order: a `Level.TransitionRoutine`-style player write in the source that the simulator's stall
path omits; the room-entry positioning the source does when the new room's bounds differ; or a solid/spring
in the new room pushing the still-collidable player while `Player.Update` is suspended. Reading `sim.rs`'s
stall branch against `Level.cs`'s transition coroutine is the cheap next step, and the verification is
binary: `--rooms roof07` should take that segment from `exact=0` to `exact=6`.

### New hypothesis: the simulator's own `freeze_timer` is > 0 while the game's is 0

The stall early-return block (`sim.rs:6301-6317`) is:

```rust
if p.death_freeze_pending { ...; p.freeze_timer = 0.05; return Ok(()); }
if p.freeze_timer > 0.0 { p.freeze_timer = (p.freeze_timer - raw_delta_time).max(0.0); }
return Ok(());
...
if p.freeze_timer > 0.0 { p.freeze_timer = (p.freeze_timer - DT).max(0.0); return Ok(()); }
```

so a positive `p.freeze_timer` is exactly what makes `step` return before `enforce_level_bounds` and before
any player update - which is what the zero probe lines showed. And the trace says `freezeTimer = 0` on every
row of that window.

Two readings, and the trace can tell them apart:

- the simulator's restored `freeze_timer` is fine and something in the replay **set** it (a `Celeste.Freeze`
  the game did not have, e.g. a dash-begin or death freeze modelled one frame off), leaving it positive for
  several frames;
- or the simulator's freeze *decrement* differs (it subtracts `DT` in one branch and `raw_delta_time` in
  another), so a freeze that the game has already left is still live in the simulator.

Note also that the trace row's `transitioning = true` means the game is in a room transition, and in the
simulator that state is reached through the `p.transition_timer > 0.0` block near the top of `step`
(`update_transition`, which does *not* return early). So if the simulator held `transition_timer > 0` it would
still be running the transition path, not the freeze return - which reinforces that the early return here is
`freeze_timer`, not a transition.

One-line probe next: print `p.freeze_timer`, `p.transition_timer`, `p.death_freeze_pending` and `p.dead` at
the top of `step` on rows 42488-42496, and compare `freeze_timer` against the trace's `freezeTimer` column.

### roof07 root cause pinned: the simulator's `freeze_timer` sits at exactly DT and never decrements

The gate's own dump prints the simulator's freeze timer per frame, so no new instrumentation was needed:

```
row=42491 gamePos=(8232,-785) gameMove=(1.00000,0.00000) rustPos=(8231,-785) rustMove=(0.00000,0.00000) stalled=true freeze=0.01667
row=42492 gamePos=(8233,-785) gameMove=(1.00000,0.00000) rustPos=(8231,-785) rustMove=(0.00000,0.00000) stalled=true freeze=0.01667
row=42493 gamePos=(8234,-785) gameMove=(1.00000,0.00000) rustPos=(8231,-785) rustMove=(0.00000,0.00000) stalled=true freeze=0.01667
row=42494 gamePos=(8235,-785) gameMove=(1.00000,0.00000) rustPos=(8231,-785) rustMove=(0.00000,0.00000) stalled=true freeze=0.01667
row=42495 gamePos=(8236,-785) gameMove=(0.58301,0.00000) rustPos=(8231,-785) rustMove=(0.00000,0.00000) stalled=true freeze=0.01667
```

The simulator's `freeze_timer` is **exactly DT (0.01667) on every frame and never decreases**, while the trace
and the dump's own `stalled=true` say the game is in a transition with `freezeTimer = 0` and is advancing the
player one pixel per frame. So the whole divergence is "the simulator holds a positive freeze timer".

Two specific places to look, both already visible in the code:

1. `sim.rs:499` - `self.snapshot.freeze_timer = self.snapshot.freeze_timer.max(DT);` - forces any
   small-but-positive value back up to DT, so a timer that should reach zero is pinned at DT;
2. the transition-nested decrement at `sim.rs:6306-6308` uses `raw_delta_time`, and the outer one at
   `:6314-6316` uses `DT`; if the first is reached with `raw_delta_time == 0` the timer never moves.

Either way the fix is small and the verification is binary: `--rooms roof07` should take that segment from
`exact=0` to `exact=6`, and `stalled=true` frames elsewhere should start updating again.

### The `stalled` witness conflates freeze with transition - the trace exports `transitioning` for exactly this

`tas_fidelity.rs:1836` defines the flag the replay uses:

```rust
stalled[index] = <trace says the engine skipped this frame>
    && stalled_row_mutates_player(&segment.frames, &truth, index);
```

and `:1855` acts on it:

```rust
if stalled[index] && !simulator_frozen { simulator.skip_engine_frame(); }
```

with `skip_engine_frame` documented as "a caller that replays a row the game skipped must not run a
`Player.Update` the game never ran". The *intent* is sound and it is why the gate matches freezes at all, but
the witness (a `Player.StrawberryCollectResetTimer` that did not move) is satisfied by **two** different
situations:

| situation | engine | player movement in the row |
| --- | --- | --- |
| `Celeste.Freeze` | `Scene.Update` skipped entirely | none - so skipping the simulator's update is right |
| room **transition** | `Level.Update` runs its transition work and entities update | **real** - `roof07` rows 42489-42494 advance the player one pixel per frame |

In the second case the flag fires and the simulator is told to skip, which is why its `freeze_timer` sits at
DT forever and the player never moves while the game creeps. `roof07` is that case: `transitioning = true`,
`freezeTimer = 0`, `stalled=true` in the same row.

The fix therefore belongs at that guard, and the discriminator the trace already carries is
`transitioning` (exported as a top-level key and used elsewhere in the harness). A transition row must not be
treated as a skipped row; if plumbing that key into the comparison is more than a line, the simulator's own
transition state is an equivalent signal, since the harness restores it.

A first attempt at the guard (`&& !stalled_row_mutates_player(...)`) was wrong twice over: the predicate is
already folded into `stalled` at `:1836`, and the call site's frame collection is `segment.frames`, not
`frames`, so it did not compile. Reverted; tree rebuilt.

### Correction: the skip guard already required `!simulator_frozen`, so the transition change is a no-op - and the timer is not decrementing

Applied the transition discriminator to the guard (`engine_skipped = stalled[index] &&
!matches!(frame.transitioning, Some(true))`) and re-ran the `--rooms roof07` slice: `exact` stayed 0 and
`frames` stayed 6, i.e. **no change at all**. The reason is in the guard itself:

```rust
if stalled[index] && !simulator_frozen { simulator.skip_engine_frame(); }
```

`simulator_frozen` was already **true** on those frames (the dump shows `freeze=0.01667`), so the second
conjunct was false and the call never fired. Adding the transition term could not change anything, and the
simulator's stuck timer has a different source. The edit is reverted rather than committed - a measured no-op
is not worth a commit even when the reasoning behind it is sound.

The sharper fact is the one the dump already carried: the timer sits at **exactly DT and never decreases**.
Its own freeze branch subtracts one `DT` per frame (`p.freeze_timer = (p.freeze_timer - DT).max(0.0)`), so
0.01667 would reach 0 in a single frame - unless that branch is not reached, or the frame's
`p.frame_delta_time` is 0, or the value is re-set every frame by something else. So the next probe belongs
on the freeze branch itself: print `p.freeze_timer` immediately before and after the decrement (and
`p.frame_delta_time`) for rows 42488-42496, and find out whether the branch runs and what it subtracts.

### The pinned DT means `step` is never called on those frames - the loop and the guard trap each other

Read the current freeze structure:

```rust
6726:  if p.freeze_timer > 0.0 { p.freeze_timer = (p.freeze_timer - raw_delta_time).max(0.0); }  // respawn block
6729:  return Ok(());
...
6734:  if p.freeze_timer > 0.0 { p.freeze_timer = (p.freeze_timer - DT).max(0.0); return Ok(()); }
```

The main freeze branch **does** subtract one `DT` and return, so a timer holding `0.01667` would reach `0` in
one call. The dump shows it pinned at exactly DT for every frame of the window, so `step` is not reaching
that branch at all - the gate's replay loop must be calling `skip_engine_frame()` **instead of** stepping on
those rows.

That closes a loop: the guard at `:1855` is `stalled[index] && !simulator_frozen`. On the first stalled row
the simulator is not yet frozen, so `skip_engine_frame()` runs and sets `freeze_timer = DT` - and from then
on `simulator_frozen` is true, so the guard's second conjunct is false and the call never fires again. If the
loop also does not step on those rows, nothing ever decrements the timer and the simulator stands still for
the rest of the window while the game advances the player. The counterexample is `roof07`, where the game's
`freezeTimer` is 0 and `transitioning` is true, so there is no freeze to model in the first place.

So the fix is not in `skip_engine_frame` and not in the freeze branch: it is in the loop's decision, which
takes one bit of information (the trace's `stalled` witness) where two are needed - *was this an engine
freeze, or a transition in which the player still moves* (`transitioning`, already parsed into `Frame`, and
already used for the diagnostic classification at `:1829`).

### The loop DOES step after skipping - so the pinned timer means an earlier return (the respawn block)

Control flow confirmed by reading the loop:

```
1856:  simulator.skip_engine_frame();     // sets freeze_timer = max(freeze_timer, DT)
1867:  match simulator.step(input) {      // still runs, every row
1868:      Ok(actual) => { ... }           // and the dump's `freeze=` column is printed here, i.e. after the step
```

So both happen on a stalled row, and the printed `freeze = 0.01667` is the **post-step** value. The main
freeze branch (`sim.rs:6734`) subtracts one `DT` and would leave 0, so `step` must be returning **before**
that branch - which leaves the respawn block:

```rust
6721:  if p.death_freeze_pending { p.death_freeze_pending = false; p.freeze_timer = 0.05; return Ok(()); }
6726:  if p.freeze_timer > 0.0 { p.freeze_timer = (p.freeze_timer - raw_delta_time).max(0.0); }
6729:  return Ok(());
```

That block is unconditional once entered (it ends in `return Ok(())`), and its decrement uses
`raw_delta_time` rather than `DT`. If the trace row's `dt` is 0, `(0.01667 - 0.0).max(0.0)` stays `0.01667`
forever - pinned exactly as observed, with the player never updated.

Two values decide it, and both are cheap to print at the top of `step` for rows 42488-42496:
`p.respawn_frames` / `p.dead` (is the respawn block even entered?) and the row's `raw_delta_time`
(the harness feeds it from the trace's `dt`, so a zero there is the smoking gun). If it is the respawn
block, the fix is that a stalled row must not enter it; if it is a zero `raw_delta_time`, the fix is in how
the harness supplies the frame delta.

### `dt` is not zero - so the pinned timer is not the respawn block either

The trace rows 42488-42496 all carry `dt = rawDt = 0.0166667` (non-zero). That kills the previous note's
hypothesis: the respawn block's `(freeze_timer - raw_delta_time).max(0.0)` would clear the timer, so the
simulator is not sitting in that block either.

Putting the known facts together:

- the loop calls `skip_engine_frame()` and then `step()` on every row (`tas_fidelity.rs:1856`, `:1867`);
- `skip_engine_frame` only raises the timer (never lowers it);
- the main freeze branch (`sim.rs:6734`) subtracts `DT` and the respawn block subtracts `raw_delta_time`,
  and `dt` is non-zero - so **any** reached decrement would clear a timer holding exactly DT;
- the dump's `freeze = 0.01667` is printed after the step, and the trace's own `freezeTimer` is 0.

So either the step returns **before both** branches (something earlier in the 6637-6720 region), or the
timer is being set again after the step by something not yet found. One probe settles it and should be the
last one in this family: at the very top of `step`, print `p.freeze_timer`, `p.dead`, `p.respawn_frames`,
`p.transition_timer`, `p.death_freeze_pending` and `raw_delta_time` **on entry**, and the same
`p.freeze_timer` **on exit** (a single `eprintln!` before the first `return` path is enough if placed at the
end of the function next to a second print at the top). That distinguishes "returns early" from "re-set
afterwards" in one run.

Note for sequencing: this family (`roof07`) has now consumed many rounds and is down to a single
instrumented question. The queue also holds the `1.2` grounded-ultra proxy and the `LaunchSpeed` lead, both
of which are one-liner candidates with existing tests to compare against.

### ROOT CAUSE for roof07: the simulator's `transition_timer` is 0 while the trace says `transitioning = true`

Entry probe at the top of `step`, every row of the window:

```
ENTRY pos=(8231,-785) freeze=0.01667 dead=false respawn=0 trans=0.0000 dfp=false state=Normal
```

Two conclusions, and the second is the answer:

1. The freeze timer is a **red herring**. `skip_engine_frame()` runs on every stalled row - the harness reads
   `simulator_frozen` *before* the step (`tas_fidelity.rs:1842`) and it is 0 there, so `!simulator_frozen` is
   true, the skip fires, and it raises the timer to DT; the step then clears it. The DT the dump shows is a
   read-point artefact, not a stuck timer.
2. **`transition_timer` is 0.0000 while the trace's `transitioning` is true.** The simulator does not know it
   is in a room transition, so its transition path never runs, the transition coroutine's player movement
   never happens, and the player stands still while the game advances it one pixel per frame. That is the
   whole of the `roof07` divergence.

Why this matters beyond one segment: the harness already parses `transitioning` (`Record`/`Frame`, used at
`:1829` for the diagnostic classification) but nothing feeds it into the simulator's transition state. Every
segment with a high `stalledFrames` count and `transitioning = true` rows is therefore replaying those rows
with no transition modelled - the `roof07` case is just the one where the harness's own accounting noticed
(`exactPrefixFrames = 0`).

Next: find how the simulator's transition state is meant to be established (the anchor restore or the
harness's stall handling) and make a `transitioning` row set it, then verify with `--rooms roof07` (both
segments should go `exact = 0` -> `6`).

### Lost work and the lesson: checkpoint a long-running writer's worktree before it can revert

The `ceiling` agent's final reports measured clean wins - 202: 7 improved / 1461 identical / **0 regressed**,
`ok` 518 unchanged, frames **160,183 -> 161,253** (+1,070); 100pct: 5 / 913 / **0**, frames
**96,228 -> 97,083** (+855) - and then its branch ended with **no commits at all** (`git -C wt-ceiling commit`
reports "nothing to commit, working tree clean"; `git merge ceiling` says "Already up to date"). So the agent
reverted its own change; presumably the third trace (`1a`, for which no report exists) or a re-measurement
regressed, but the code is gone and only the two reports remain.

That is a real loss of a +1,070-frame variant, and the cause is a process gap on my side: for `gate` I
committed the agent's worktree to its branch **myself** before it could revert, which is the only reason that
+5-segment win survived. The rule to keep: when a writer subagent has uncommitted work that measures clean on
any trace, commit it to its branch as a checkpoint - it costs nothing, the branch is not master, and a later
`regressed != 0` on another trace can still be handled by not merging it.

The variant itself is documented well enough to rebuild if it turns out to be wanted: the `on_ground` proxy on
the 1.2x grounded-ultra branch (`sim.rs:7718`) needs to become a "downward move was blocked this frame" signal
taken from the collision path, and the ceiling case in `Player.WindMove` needs the source's response rather
than `move_axis_amount`'s.

### FOUND, by the ceiling workstream: `WindMove`'s moves have a null collide callback

The agent's own comment, from the checkpoint `0253a72` on branch `ceiling` (34 lines, `sim.rs` only):

> `Player.WindMove` moves with `MoveH(move.X)` / `MoveV(move.Y)` (`Player.cs:3107`, `:3132`), both with the
> default `onCollide = null` (`Actor.cs:186-208`). `MoveVExact`/`MoveHExact` still probe
> `CollideFirst<Solid>` and still clear `movementCounter` on the blocked step (`Actor.cs:220`, `:249`), but
> `onCollide?.Invoke(...)` is skipped, so `Player.OnCollideH`/`Player.OnCollideV` never run and `Speed` is
> left untouched.

That is the missing link in the updraft chain: an upward wind step that is blocked by a ceiling does **not**
zero `Speed.Y` in the game, while `move_axis_amount` (which always runs the collide path) does zero it in the
simulator - which is exactly the 7.5 -> 0 that started this investigation. The implementation adds
`move_axis_amount_inner(..., collide: bool)` and a `move_axis_amount_silent` wrapper used by the wind.

Its measurements before reverting were clean on two traces: 202 `7 / 1461 / 0` with frames
`160,183 -> 161,253`, 100pct `5 / 913 / 0` with `96,228 -> 97,083`; the third trace (1a) had no report, and
the agent reverted before committing. **The checkpoint `0253a72` preserves the work**, so the remaining step
is just: run the three traces (1a included) on that branch and merge if all three are `regressed=0`.

This is also the concrete instance of the checkpoint rule added above - the work would otherwise have been
lost twice.

### LANDED: the wind's callback-less moves (master `b2d3887`)

Independently verified on all three traces from the merged tree, reproducing the workstream's numbers exactly:

| trace | improved / identical / regressed | ok | frames | exact |
| --- | --- | ---: | ---: | ---: |
| 202 | 7 / 1461 / **0** | 518 | 160,183 -> **161,253** | 159,203 -> **160,273** |
| 100pct | 5 / 913 / **0** | 339 | 96,228 -> **97,083** | 95,632 -> **96,487** |
| 1a | 0 / 20 / **0** | 16 | 2,129 unchanged | 2,125 unchanged |

354 tests pass. `7-Summit|0|g-01|134447` goes from 833 to **1198** replayed frames (row 135320 now matches
bit for bit), and the seven improved segments are all wind rooms (`4-GoldenRidge|0|a-09`, `c-04` x2,
`7-Summit|0|g-00b` x2, `g-01`, `e-10`). `ok` is flat because each of those segments now diverges later for a
different reason.

Two process notes worth keeping:

- the 1a baseline is `gate-tg2-1a.json` (or the workstream's `ceil-base-1a.json`); `gate-fl2-1a.json` does
  not exist, and pointing `q-regress` at it fails with `ENOENT` after the run has already produced its
  numbers - read the printed totals in that case, they were 16/2129/2125, unchanged.
- the workstream also flagged that `Copy-Item` preserves mtimes, so restoring a file with it does **not**
  retrigger cargo; it had to `touch` the file to get a real rebuild (hashes differed). That is the same
  stale-binary trap this document warns about, in a new disguise.

### `roof07` / transition rows: state of play after the workstream stalled

The `transition` workstream (worktree `wt-transition`, branch `transition`) ran a 202 baseline and then
produced nothing further across several rounds - no code changes, no slice report - so it was stopped. Its
worktree is clean, so nothing was lost, but the round spent on it produced no result.

What the project already knows about the case, from measurements in these notes:

- the replay treats the rows as stalled and the simulator's `transition_timer` is **0** while the trace says
  `transitioning = true`, so the simulator never runs a transition model there (`eb1c63c`);
- the game advances the player exactly **+1 px per frame** with `vx` constant (nothing writes `Speed`),
  stopping when the collider's left edge lands exactly on the room's left bound (`roof07`: `Bounds.Left = 8232`,
  collider `[8232,-796,8,11]`) - the shape of a per-frame push or exact move, not of ordinary physics;
- `Level.EnforceBounds`'s own clamp is a **hard set** (`player.Left = bounds.Left`, `Level.cs:2746`), which
  cannot by itself produce a one-pixel creep, so either the game's clamp runs *before* the move each frame or
  the transition coroutine is what moves the player toward the room;
- the trace carries only the boolean `transitioning` (`Frame.transitioning`, already parsed and used at
  `tas_fidelity.rs:1829`), not the transition's target or duration - so the fix has to let the simulator's
  own transition model run rather than replay the movement from the trace.

Success is still binary and cheap: `--rooms roof07` should take both `roof07` segments from `exact = 0` to
`exact = 6`. The baselines to compare against are now `ceilc-202.json` (518 ok / 161,253 frames / 160,273),
`ceilc-100pct.json` (339 / 97,083 / 96,487) and `gate-tg2-1a.json` (16 / 2,129 / 2,125).

Sequencing note: two workstreams have now stalled on this case (the first was my own multi-round
investigation). It is well characterized but it is also demonstrably hard for a subagent briefed from
documentation alone, so the next attempt should be either a very small in-session step (one dump, one
decision, one edit) or a brief that starts from the two shapes and names the single command to run first.

### FOUND the roof07 creep: `Player.TransitionTo` moves at exactly 60 px/s

`Player.cs:2293-2308`:

```csharp
public bool TransitionTo(Vector2 target, Vector2 direction) {
    MoveTowardsX(target.X, 60f * Engine.DeltaTime);
    MoveTowardsY(target.Y, 60f * Engine.DeltaTime);
    UpdateHair(applyGravity: false);
    UpdateCarry();
    if (Position == target) { ZeroRemainderX(); ZeroRemainderY(); Speed.X = (int)Math.Round(Speed.X); ... return true; }
    return false;
}
```

`60 * dt` is exactly **one pixel per frame**, which is the creep measured on rows 42489-42494 (+1 px, `vx`
constant because `MoveTowards` never writes `Speed`), and the stop condition `Position == target` is why it
halts at `x = 8236` and stays there. So the mechanism is the transition coroutine moving the player toward
its target, and the simulator's `transition_timer` being 0 is why it does nothing at all.

What the simulator still needs is the **target** (`Level.NextLevel` computes it from the destination room's
entry and passes it to the transition routine); it is not in the trace, but the destination room is - it is
the room of the following segment - and the harness already decodes every room's map, so the target is
derivable rather than guessable. That is a small, self-contained piece of work now: implement
`MoveTowards(target, 60 * dt)` in the transition path, aim it at the destination room's entry position, and
verify with `--rooms roof07` (both segments should go from `exact = 0` to `exact = 6`).

This supersedes the two-shape guessing in the previous note: the dump already showed the game's collider
landing exactly on `Bounds.Left`, and `TransitionTo` explains both the one-pixel rate and the stop.

### roof07: the machinery exists - only the anchor-side transition state is missing

Everything is already implemented:

- `TRANSITION_MOVE_SPEED = 60.0` (`sim.rs:117`), matching `Player.TransitionTo`'s `60f * Engine.DeltaTime`
  (`Player.cs:2295-2296`) - one pixel per frame;
- `begin_transition` sets `p.transition_target = target` (`:11298`) and
  `p.transition_timer = TRANSITION_TIME + p.frame_delta_time` (`:11303`);
- `update_transition` (`:11306`) does the `approach(pos, target, 60*dt)` and the
  remainder/speed-rounding at arrival.

So the whole gap is that a replayed window which **starts inside a transition** has `transition_timer = 0`
(the entry probe shows `trans=0.0000`) and never enters `update_transition`. Neither `transition_timer` nor
`transition_target` is exported by the trace, but the trace *does* say `transitioning = true` on those rows -
so the harness can synthesise the state at the anchor, exactly as it does for the other unexported session
state.

The target is derivable: `Level.EnforceBounds` calls `NextLevel(player.Center + UnitX * 8f, UnitX)` for a side
transition (read in an earlier round), so the direction is readable from the anchor geometry (the player is
outside the room on its left, so the transition is to the right) and the target is a function of that. One
caveat to check while implementing: the measured stop is `x = 8236`, whereas `center + 8` from the anchor
position (`x ~ 8230`, ducking collider 8 wide, so center `8234`) would be `8242` - a six-pixel difference, so
`NextLevel` adjusts the target beyond that expression and the implementation should print the target it
computes against the observed stop before trusting it.

Verification stays binary: `--rooms roof07`, both segments `exact = 0` -> `exact = 6`.

### The transition bit must be ANDed with the stalled witness (the trap that cost 15 segments their last frame)

From the `transition` workstream, and it is the same trap for any future transition work:

> `Level.Transitioning` becomes true on the frame the `TransitionRoutine` is **created** - inside the
> `Player.Update` that ran `Level.EnforceBounds` - while Monocle only resumes a fresh coroutine on the
> **next** `Update`. So the transition's first row is an ordinary `Player.Update` row, and it is the **last
> row of the segment for the room being left**.

Gating on the trace's `transitioning` bit alone therefore cost **every room-change segment its final frame**:
15 segments went `ok` -> `mismatch` with frames unchanged and `exact` one lower each. The fix is to treat a row
as a transition frame only when `transitioning` **and** the harness's own stalled witness (an unchanged
`Player.StrawberryCollectResetTimer`) both hold - i.e. the bit says a transition exists, the witness says the
player was not updated by the game this frame.

With that: `--rooms roof07` is `ok=2 mismatch=0 frames=114 exact=114` (from `ok=0 mismatch=2 frames=12
exact=0`), and the 1a diff against a same-code baseline is `0 / 20 / 0` (2,129 frames, 2,125 exact, identical
to `gate-tg2-1a.json`). The full 202 and 100pct runs were still in flight at the time of writing.

The general lesson: a boolean from the trace that describes a *level* state can lead the *player* state by a
frame, because Monocle's coroutine resumption is deferred. Any replay logic keyed on such a bit needs the
player-side witness alongside it - which is exactly what the harness's stalled accounting already computes.

### LANDED: room transitions are replayed (master `e17738d`, branch tip `f33314e`)

Verified on the merged tree against this document's baselines, all three traces `regressed=0`:

| trace | improved / identical / regressed | ok | frames | exact |
| --- | --- | ---: | ---: | ---: |
| 202 | 2 / 1466 / **0** | 518 -> **520** | 161,253 -> **161,355** | 160,273 -> **160,387** |
| 100pct | 1 / 917 / **0** | 339 -> **340** | 97,083 -> **97,134** | 96,487 -> **96,544** |
| 1a | 0 / 20 / **0** | 16 | 2,129 | 2,125 |

`cargo test --release -p celeste-physics --lib` is **356 passed / 0 failed** (one new test). Two mechanisms
landed together:

1. **`transitioning` gating ANDed with the stalled witness** - a level-level boolean leads the player by a
   frame, because `Level.Transitioning` turns true on the frame `TransitionRoutine` is *created* while Monocle
   resumes the coroutine only on the next `Update`. Using the bit alone cost every room-change segment its last
   frame (15 segments `ok` -> `mismatch`); ANDing it with an unchanged
   `Player.StrawberryCollectResetTimer` fixes that.
2. **The anchor-side transition state** - a replayed window can start inside a transition, and neither
   `transition_timer` nor `transition_target` is exported, so the state is synthesised. The target comes from
   `Level.TransitionRoutine`'s walk seeded on the bound being entered, and the walk is **asymmetric**:
   `Left + 4` travelling right, `Right - 5` travelling left, `Top + 12` down, `Bottom - 5` up - which
   `begin_transition` already encodes and the seeded walk now reproduces term for term.

Measured robustness note worth keeping: over `trace-202-v7.jsonl`, 2,874 of 438,001 level rows have a
fractional `Player.Position` (all `StCassetteFly`-style tweens) and **none** of its 54,521 `transitioning`
rows do - so a bound-derived target and a start-derived target agree on every replayed transition row. That is
also why the target is invariant under whole-pixel offsets: `start + ceil(bound - start)` only carries the
start's fractional part.

Process notes: the preliminary verification of this work was done in a detached worktree at a checkpoint
commit, because the workstream had gone quiet with its edits uncommitted; the first merge attempt failed at
`cargo test` (the workstream's own new test contained a self-contradictory assertion) and master was reset to
green rather than pushing a red tree - the landing script now gates on the test result as well as on
`regressed`, which is what this document's earlier discipline was missing.

### `LaunchSpeed` (±280): the simulator's `explode_launch` matches the source term for term

Read both sides (`Player.ExplodeLaunch`, `Player.cs:4919-4955`; `sim.rs` `explode_launch`, `10832-10879`):

| source | simulator |
| --- | --- |
| `Celeste.Freeze(0.1f)` | `p.freeze_timer = 0.1` |
| `launchApproachX = null` | `p.launch_approach_x = None` |
| `(Center - from).SafeNormalize(-UnitY)` | `normalize(delta)`, `(0,-1)` when `delta == 0` |
| `snapUp && num <= -0.7` -> `(0,-1)` | same |
| `num <= 0.65 && num >= -0.55` -> `(sign(x),0)` | `(-0.55..=0.65)` range |
| `sidesOnly && x != 0` -> `(sign(x),0)` | same |
| `Speed = 280f * vector` | `scale(direction, 280.0)` (+ unit test asserting `(-280,-150)`) |
| `Speed.Y <= 50f` -> `Math.Min(-150f, ...)` + `AutoJump` | `min(-150.0)` + `auto_jump` |
| `Speed.X != 0f` -> 1.2x / `explodeLaunchBoostTimer` split | same, both branches |
| `if (!Inventory.NoRefills) RefillDash()` + `Stamina = 110` | same |

So the whole value path is correct and the +-280 cluster is a **frame-placement** question, exactly as the
round-124 note concluded - but now the numeric path is excluded term by term rather than assumed.

The remaining question is the **trigger frame**, i.e. the callers. The simulator calls `explode_launch` from
two places (`sim.rs:10601` with `snap_up=false, sides_only=false`; `:10726` with `sides_only=true`), so the
next step is to list the source's callers of `ExplodeLaunch` (lava, the `Bumper`-family, `Badeline`'s
projectile launch) and check which of them the +-280 segments exercise - the odd one out is where the frame
difference lives. Note the stale line reference in the earlier note (`sim.rs:10764` is now
`update_strawberry_train`): the launch moved to `10832`, so prefer name anchors over line numbers in this
file.

### `ExplodeLaunch`: the simulator implements two of the four player-facing callers

Callers on both sides:

| source | arguments | simulator |
| --- | --- | --- |
| `Bumper.cs:170` | `ExplodeLaunch(Position, snapUp: false)` | `sim.rs:10601` `(snap_up=false, sides_only=false)` |
| `Puffer.cs:233` | `ExplodeLaunch(Position, snapUp: false, sidesOnly: true)` | `sim.rs:10726` `(false, true)` |
| **`Seeker.cs:1042`** | `ExplodeLaunch(Position)` -> defaults, so **`snapUp = true`** | **missing** |
| **`TempleBigEyeball.cs:66`** | `ExplodeLaunch(player.Center + UnitX * 20f)` -> **`snapUp = true`** | **missing** |
| `Puffer.cs:238`, `Seeker.cs:1047` | `theoCrystal.ExplodeLaunch(...)` | separate (Theo) family |

Two of the four player-facing callers are absent, and both are precisely the ones that take the **default
`snapUp = true`** - which is the branch that snaps the direction to straight up `(0,-1)`, giving
`Speed.Y = -280` and then the `Speed.Y <= 50f` clamp to **-150** with `AutoJump = true`. That signature is
exactly what the +-280 cluster shows, so these two missing triggers are the best candidate for it.

Next: check whether the segments in that cluster sit in rooms that contain a `Seeker` (MirrorTemple,
Reflection) or a `TempleBigEyeball`, and whether the simulator has entity kinds for them at all - if it does
not, the fix is a new trigger plus the entity's own behaviour, which is a larger piece of work than the
one-liners landed so far.

### The missing `ExplodeLaunch` is `Seeker.RegenerateCoroutine`'s final push-away

The source site (`Seeker.cs:1028-1048`):

```csharp
private IEnumerator RegenerateCoroutine() {
    yield return 1f;                       // 1.00 s
    shaker.On = true;
    yield return 0.2f;                     // 0.20 s
    sprite.Play("pulse");
    yield return 0.5f;                     // 0.50 s
    sprite.Play("recover");
    RecoverBlast.Spawn(Position);
    yield return 0.15f;                    // 0.15 s
    base.Collider = pushRadius;
    Player player = CollideFirst<Player>();
    if (player != null && !base.Scene.CollideCheck<Solid>(Position, player.Center))
        player.ExplodeLaunch(Position);    // snapUp defaults to true
    ...
}
```

So the missing trigger is a **regeneration coroutine**: after a Seeker is hit it enters state 4 (Stunned), regenerates over roughly 1.85 s of yields, and at the end - if the player is still inside its `pushRadius` and no solid sits between the Seeker and the player's centre - it launches the player away. The simulator's state-4 handling (`sim.rs:3194-3202`) only counts `state_timer` down and returns to state 0, so the whole coroutine is skipped and the launch never happens. That is why the +-280 signature appears in Seeker rooms (`5-MirrorTemple`, `6-Reflection`) and `ExplodeLaunch`'s `snapUp` default of true matters: the launch snaps to straight up, giving `Speed.Y = -280` clamped to `-150` with `AutoJump`.

Implementation sketch: give `SeekerSnapshot` the coroutine phases (the 1.0 / 0.2 / 0.5 / 0.15 s yields are expressible in `state_timer` values, as other states already do), then at the final phase set `collider = pushRadius`, test the player overlap and the solid-between check, and call `explode_launch(p, input, seeker.position, true, false)` - i.e. with the source's default `snapUp = true`, which is precisely the branch the simulator has never exercised.

`TempleBigEyeball` (`TempleBigEyeball.cs:66`) is the other `snapUp = true` caller and has **no** entity kind in the simulator at all, so it is a larger piece of work; the Seeker is the one to do first because the entity, its states, its stun and its bounce are already modelled.

### Seeker launch: the last facts needed to implement it mechanically

`SeekerSnapshot` and the stun/bounce model already exist; what the regeneration launch needs is:

- **`pushRadius = new Circle(40f)`** (`Seeker.cs:265`) - a **circle** of radius 40 centred on the Seeker, not a
  rect, and at the coroutine's end the Seeker's `Collider` is set to it, so `CollideFirst<Player>()` is a
  circle-versus-player-hitbox test. The simulator's existing `seeker_attack_rect` (`sim.rs:3088`) and
  `seeker_bounce_rect` (`:3092`) mirror `attackHitbox = (12,8,-6,-2)` and `bounceHitbox = (16,6,-8,-8)`, so a
  `seeker_push_overlaps(position, player_rect)` helper belongs beside them.
- **`!Scene.CollideCheck<Solid>(Position, player.Center)`** (`Seeker.cs:1040`) - a solid test along the
  *segment* from the Seeker to the player's centre. The simulator has **no** such helper (grep finds no
  `line_of_sight`/`collide_check_between`), so this is the one genuinely new piece: walk the segment against
  `map.solids` (and, if the codebase's Badeline/Puffer paths do something equivalent, match their approach).
- **The phases** are the four yields `1.0 / 0.2 / 0.5 / 0.15` s (`Seeker.cs:1030-1037`), which `state_timer`
  can carry the way other timed states in `sim.rs` already do; the launch belongs to the last phase, after
  `base.Collider = pushRadius`.
- **The call is `explode_launch(p, input, seeker.position, true, false)`** - `snapUp = true` is the source's
  default and is the branch the simulator has never exercised, which is why the +-280 signature shows up in
  Seeker rooms.

Workstream status: branch `seeker` still has zero changes after three rounds; nothing is lost (worktree
clean), and the task is now specified to the point where it is a mechanical edit rather than a diagnosis.

### Correction: the +-280 cluster is NOT the Seeker - measured entity distribution

The `seeker` workstream checked every room the 202 trace visits with `inspect_map` and found that in this map
set Seekers live **only** in `Celeste/5-MirrorTemple` (rooms `c-*`, `d-*`, `e-*`, `void`) and
`Celeste/LostLevels` (Farewell: `c-00`, `c-alt-00`, `d-00`, `e-02/04/06/08`, `j-*`, `end-golden`).
**`6-Reflection` contains no Seeker entity at all**, and neither do `9-Core` or `LostLevels e-00`. Every
first-mismatch row of the +-280 cluster that lives in Reflection (`a-05`, `15`, `b-03`, `a-04`, `10a`,
`b-02`), Core (`c-00b`, `c-03`, `01`) or LostLevels `e-00` therefore **cannot** be a Seeker push-away, and in
MirrorTemple no segment's first mismatch is a `StLaunch` at all.

So the previous note's inference - "the signature matches, so it is the Seeker" - is wrong. The cluster points
at the **other `snapUp = true` caller, `TempleBigEyeball`** (`TempleBigEyeball.cs:66`), or at the existing
Puffer/Bumper trigger **frame placement** - not at the Seeker. That is the second time in this investigation
that a plausible signature-based inference had to be retired in favour of counting the entities (the first was
the phantom-solid reading, which a downward-only probe could not see).

Independently measured while testing an interim Seeker patch of my own (since discarded): the Seeker
stun-exit change is **inert on all three traces** - `0 / 1468 / 0`, `0 / 918 / 0`, `0 / 20 / 0` with frames and
`exact` unchanged - so the Seeker stun-exit path is not what the corpus is missing either. The workstream's own
version (`98c9968` on branch `seeker`) models the whole `RegenerateCoroutine` phase chain and is unit-tested;
whether it earns a landing depends on its three-trace diff, not on the cluster story.

Also worth keeping: `--rooms` matches `segment.room`, which is a **room-local** string like `a-00` or `15` - not
a chapter name. `--rooms "5-MirrorTemple"` selects nothing (`simulated=0`, `skipped=1468`), which is a silent
no-op that looks like a passing slice.

### +-280 follow-up: the Bumper trigger is equivalent, so the candidates narrow to Puffer and TempleBigEyeball

Read both sides:

| source `Bumper.cs:159-170` | simulator `sim.rs:10594-10605` |
| --- | --- |
| `else if (respawnTimer <= 0f)` (after the collide gate) | `if p.bumpers[index].respawn_timer <= 0.0` |
| `respawnTimer = 0.6f` | `p.bumpers[index].respawn_timer = 0.6` |
| `Vector2 vector2 = player.ExplodeLaunch(Position, snapUp: false)` | `explode_launch(p, input, target, false, false)` with `target` = the Bumper's centre |

Same trigger condition, same flag (`snapUp = false`), same respawn constant, same direction source
(`Position`). So the Bumper is **not** a frame-placement candidate for the +-280 cluster. That leaves:

1. the **Puffer** path (`Puffer.cs:233`, `snapUp: false, sidesOnly: true`; `sim.rs:10726` already passes
   `(false, true)`), where the remaining question is the same overlap-gate frame; and
2. **`TempleBigEyeball`** (`TempleBigEyeball.cs:66`, the default `snapUp = true`), which the simulator does not
   model at all.

Given (1) is cheap to read and (2) is a new entity, the next step is to compare the Puffer's collide gate on
both sides before considering (2).

### +-280 candidate: the Puffer's launch lives in `Explode()`, the simulator fires it on overlap

Source (`Puffer.cs:224-233`):

```csharp
private void Explode() {
    Collider collider = base.Collider;
    base.Collider = pushRadius;                      // circle, same shape as the Seeker's
    ...
    Player player = CollideFirst<Player>();
    if (player != null && !base.Scene.CollideCheck<Solid>(Position, player.Center))
        player.ExplodeLaunch(Position, snapUp: false, sidesOnly: true);
}
```

So the Puffer's launch is **not** an overlap interaction: it happens inside `Explode()`, i.e. when the Puffer
transitions into its exploding/gone state (after being hit and running its own state machine), and it is gated
on the `pushRadius` circle plus the no-solid-between segment check - the same shape as the Seeker's
`RegenerateCoroutine` tail.

The simulator calls `explode_launch(p, input, target, false, true)` from a per-entity interaction switch
(`sim.rs:10726`, the same level as the Bumper's `:10601`), which is an **overlap-time** trigger. That is a
frame-placement difference of exactly the kind the +-280 cluster shows, and unlike `TempleBigEyeball` the
Puffer entity, its states and its motion are already modelled - so this is the cheapest remaining candidate.

Next: read the simulator's Puffer branch and its state machine (`Gassed`/`Gone`/`goneTimer`, `Puffer.cs`
`Update`) to see whether the explosion is already modelled elsewhere and merely fires the launch too early,
then move the launch to the explosion transition and re-measure. The gate itself needs the `pushRadius`
circle and the segment check - the `segment_hits_solid` helper written for the Seeker work (branch `seeker`)
is directly reusable if that branch lands; otherwise it is eight lines.

### +-280: the Puffer has NO state in the simulator, and the source gates the explode twice

`Explode()` is called from the collide handling of `States.Idle` (`Puffer.cs:394`) and `States.Hit` (`:422`),
and two guards stand in front of it:

- `:296` / the collide path: `if (num > 0f && state != States.Gone)` - a Puffer in `Gone` does not explode;
- **`:552`: `if (state == States.Gone || !(cantExplodeTimer <= 0f)) return;`** - and `cantExplodeTimer` is set
  to **0.5 s** at `:167` (on (re)spawn) and only ticks down while `state != States.Gone` (`:362-365`), with a
  second copy of the same gate at `:506`.

The simulator models **none** of this: grep for `cant_explode`, `gone_timer` or `PufferSnapshot` in `sim.rs`
and `types.rs` returns nothing. Its only Puffer handling is the `else` branch of the stomp test
(`sim.rs:10725-10728`), which calls `explode_launch(p, input, target, false, true)` unconditionally on a
non-stomp collision and mirrors `goneTimer` only as `bounce_reuse_timer = 2.5`. So the simulator explodes a
Puffer on collisions where the game refuses (a Puffer that has just respawned, or one in `Gone`), which is
exactly a one-frame placement difference in the +-280 cluster.

Fix sketch: give the Puffer per-entity state (`state` + `cant_explode_timer`, set to 0.5 on spawn, ticking
down while not `Gone`), add the `state != Gone && cant_explode_timer <= 0` guard to both the collide path and
the launch, and keep the existing `2.5` s gone timer as `bounce_reuse_timer`'s sibling. That is small: one
field pair, one decrement site and one guard, all at an already-modelled entity.

### Seeker work: inert on 202 by two independent implementations - not landing

The `seeker` workstream's own after-run on `trace-202-v7.jsonl` is byte-identical to master's
(`0 / 1468 / 0`, `520 / 161,355 / 160,387`), which matches what my discarded interim patch measured
(`0 / 1468 / 0`). Two independent implementations of the Seeker regeneration push-away therefore agree that
the path is inert on this corpus.

Combined with round 184's entity-distribution measurement - Seekers exist only in `5-MirrorTemple` and
`LostLevels`, `6-Reflection` has none, and in MirrorTemple no segment's first mismatch is a `StLaunch` - the
work's motivating hypothesis is dead, and landing it would add an unverified timing approximation
(the coroutine phase chain) for no measurable gain. The branch `seeker` keeps the work as a record
(checkpoints `98c9968`, `33f0ef8`); `master` stays as is. The workstream has been stopped so the machine is
free for the Puffer work, which is the live candidate.

That is the third time in this session that a signature-based hypothesis was retired by counting entities or
by measuring inertness rather than by landing a plausible-looking fix.

### Puffer work: an unexplained `E0119`, and a claim in this log that needs re-checking

Attempted the first half of the Puffer fix (add `PufferSnapshot`, a `puffers: Vec<...>` field and its manual
`Default` entry to `types.rs`). The build failed with

```
error[E0119]: conflicting implementations of trait `Clone` for type `PufferSnapshot`
error[E0119]: conflicting implementations of trait `Debug` for type `PufferSnapshot`
error[E0119]: conflicting implementations of trait `StructuralPartialEq` for type `PufferSnapshot`
error[E0119]: conflicting implementations of trait `PartialEq` for type `PufferSnapshot`
```

which is exactly the derive list my patch inserted - as if the type already existed. The revert ran, and a
re-check of the *restored* file finds **no** `PufferSnapshot`, `puffers` or `cant_explode` anywhere in
`types.rs` **or** `sim.rs`. So the failure is explained neither by a pre-existing definition nor by a double
insertion in that patch (the replacement text contains the anchor once), and I am recording it as unexplained
rather than guessing. The next attempt should grep for `PufferSnapshot` and insert only if it is absent, and
should add the type **without** derives first, letting the compiler ask for what it needs.

Related correction: the earlier note said "the simulator models **no** Puffer state, grep for `cant_explode`,
`gone_timer`, `PufferSnapshot` in `sim.rs` and `types.rs` returns nothing". That grep in fact covered `sim.rs`
only, and this round's re-check of both files still finds nothing - so the conclusion may well stand, but the
claim was made from weaker evidence than it stated and should be re-verified before the fix is designed around
it. Nothing in `crates/` is modified right now.

### LANDED: the Puffer explode gates (master `27a6649`) - inert on this corpus, correct per the source

`Puffer.cs` refuses to explode while the Puffer is `Gone` (`:296`, `:552`) or inside the 0.5 s spawn cooldown
`cantExplodeTimer` (`:167`, ticked down at `:362-365`). The simulator modelled **no** Puffer state at all
(verified by searching every field name in `types.rs` and `sim.rs`, not just one file), so it launched the
player on any non-stomp collision. This lands:

- `types.rs`: `PufferSnapshot { state, cant_explode_timer, center }`, a `puffers` vector and its manual
  `Default` entry;
- `lib.rs`: `PufferSnapshot` added to the explicit `pub use types::{...}` list - that list is explicit, which
  is why the first build failed with "cannot find struct in the crate root";
- `sim.rs`: `initialize_puffers` (spawn `state = 0`, `cant_explode_timer = 0.5`, centre from the entity
  bounds) wired at both `initialize_seekers` call sites, `advance_puffers` (ticks the timer down while
  `state != Gone`) called each frame next to `advance_seekers`, and the gate
  `state != Gone && cant_explode_timer <= 0.0` on the launch, looked up by centre so no index has to be
  threaded into the interaction loop.

Measured verdict: **inert on all three traces** - `0 / 1468 / 0`, `0 / 918 / 0`, `0 / 20 / 0`, with ok,
frames and exact unchanged (202 = 520 / 161,355 / 160,387; 100pct = 340 / 97,134 / 96,544; 1a = 16 / 2,129 /
2,125) and 356 tests green. So the +-280 cluster does **not** come from this either, and `TempleBigEyeball`
(which has no entity kind at all) is the remaining `snapUp = true` candidate, together with whatever else
moves those rows.

Landing an inert-but-correct change follows the `Session.DoNotLoad` precedent in this log (`822134a`, also
measured `0 / 1468 / 0`): the mechanic is faithfully modelled, the corpus simply does not exercise it, and the
alternative - leaving a known modelling gap in place - is worse for the next corpus.

Two process notes for the record: the go/no-go script now gates on **both** the `test result` line and the
three `regressed` counts (a missing test gate nearly pushed a red tree earlier), and every insertion in this
file must use `\r?\n` regex anchors - `[char]10` anchors silently fail on this CRLF checkout, which cost an
attempt this stretch.

### Where the remaining mismatches live: the Summit updraft shaft, and a named residual there

Ranking the 949 mismatching segments by room and total `stalledFrames` (from the current report, `puf-202.json`):

| room | segments | frames | exactGap | stalled |
| --- | ---: | ---: | ---: | ---: |
| `7-Summit\|g-01` | 4 | 2,523 | 4 | **1,034** |
| `6-Reflection\|02` | 4 | 276 | 4 | 706 |
| `7-Summit\|g-03` | 4 | 152 | 4 | 615 |
| `7-Summit\|g-02` | 4 | 888 | 4 | 558 |
| `4-GoldenRidge\|c-03` | 2 | 288 | 2 | 528 |
| `7-Summit\|g-00b` | 3 | 1,153 | 3 | 521 |
| `7-Summit\|g-00` | 4 | 2,656 | 4 | 420 |
| `7-Summit\|e-01` | 2 | 260 | 2 | 413 |
| `9-Core\|space` | 4 | 72 | 4 | 406 |
| `3-CelestialResort\|09-b` | 6 | 648 | 6 | 378 |
| `5-MirrorTemple\|b-22` | 1 | 27 | 1 | 370 |

Two readings:

- `stalledFrames` routinely exceeds `frames` (most extreme: `b-22`, 370 against 27). That is the counter's
  scope - it counts stalled rows across the segment's window, while `frames` counts replayed frames - not a
  defect, and it means the column is a measure of *paused time near the segment*, useful for ranking but not
  comparable to `frames`.
- `exactGap == segment count` in nearly every row, i.e. one missing frame per segment, which is the harness
  stopping at the first divergence - the tautology already documented. The informative part is the room
  ranking, and it points at the **Summit updraft shaft** (`g-00`, `g-00b`, `g-01`, `g-02`, `g-03`), which is
  the same family the wind fix came from.

Concrete named lead, left by the wind workstream's own closing report: `7-Summit|0|g-01|134447` now diverges at
its **new** tail - offset 1196 / row 135684 has `gameCounter.y = 0.45838` against the simulator's `0.0` with
positions still equal, and the following row has `pos.y` off by one (`-19385` vs `-19386`) with speeds in
agreement. That is the cheapest next target: a vertical movement-counter difference in an updraft room, one
frame before a one-pixel position difference.

### g-01 new tail: a bounce whose vertical movementCounter the simulator clears and the game keeps

Dump of `7-Summit|0|g-01|134447` around the new divergence:

```
offset=1195  gameCounter=(-0.43004, 0.37504)  rustCounter=(-0.43004, 0.37504)   identical
offset=1196  gameCounter=( 0.29774, 0.45838)  rustCounter=( 0.29774, 0.00000)   y differs
             gameMove   =( 1.72778,-1.91666)  rustMove   =( 1.72778,-2.37504)   differ by 0.45838
             gameSpeed  =( 0.00000,-185.00000) rustSpeed =( 0.00000,-185.00000)  identical
             gamePos    == rustPos            =(26363,-19382)                   identical
offset=1197  gamePos=(26363,-19385)           rustPos=(26363,-19386)           1 px apart
```

Readings:

- `-185` is `SuperBounce`'s speed, and both sides already agree on it at offset 1196, so a bounce happened
  on that frame on both sides;
- the positions agree too, but the **vertical movement counter** does not: the game carries a `0.45838`
  fraction out of the frame while the simulator has cleared it, and the difference between the two `Move`
  totals is exactly that same `0.45838`. So the simulator **zeroed a remainder the game kept** - the signature
  of a blocked move whose collide path clears `movementCounter` (`Actor.cs:220`/`:249`), which is exactly what
  `move_axis_amount` does when it reports `collided`;
- one frame later that becomes a one-pixel position difference, and nothing else diverges.

This is very likely the third instance of the class the wind workstream listed as deliberately untouched:
"same class, NOT exercised by these traces" - `SuperBounce`/`SideBounce` `MoveV` (`sim.rs` around
`11348/11367/11378`), the `JumpThru` assist (`:7032`) and `MoonLanding`. The wind fix showed the shape: those
source moves call `MoveV`/`MoveH` **without** a collide callback, so a blocked whole-pixel step clears
`movementCounter` but must **not** run the `OnCollide` response. Here the remainder is being cleared
differently between the two sides, so the next step is to read `Player.SuperBounce`/`SideBounce` and the
simulator's bounce path and compare which one clears the counter - the fix is likely the same
`move_axis_amount_silent`-style split already landed for the wind.

### The g-01 bounce: the source swaps in `normalHitbox` before a callback-less `MoveV`

`Player.SuperBounce` (`Player.cs:2708-2739`):

```csharp
Collider collider = base.Collider;
base.Collider = normalHitbox;        // tall 8x11 box, temporarily
MoveV(fromY - base.Bottom);          // MoveV with the default onCollide = null
...
varJumpSpeed = (Speed.Y = -185f);
```

and `SideBounce` (`:2741-2766`) is the same shape: swap to `normalHitbox`, `MoveV(Calc.Clamp(fromY -
base.Bottom, -4f, 4f))`, then optionally `MoveH`.

This refines the previous note's reading. A **callback-less move still clears `movementCounter` when it is
blocked** (`Actor.cs:249`, and the wind fix proved the simulator already models that), so the divergence at
offset 1196 is not "who clears the counter" but **which of the two moves was blocked at all**: the game's was
not (it carries `0.45838` out of the frame) and the simulator's was (its counter is exactly `0`). The likely
reason is right there in the source - the move happens with `normalHitbox` swapped in, so any difference in
which rectangle the simulator uses for that step (ducking box, hurt box, or the ordinary hitbox) decides
whether the step collides. One pixel of position difference on the following frame is the consequence.

Next: read the simulator's `bounce`/`reset_for_spring_bounce` path and check which rect it passes to the
movement, then make the bounce swap to the normal hitbox exactly as the source does.

### The bounce rect is right; the arithmetic says something ELSE cleared the simulator's counter

`player_rect` (`sim.rs:9543`) is `Rect::new(x - 4.0, y - 11.0, 8.0, 11.0)` - the ordinary 8x11 body - and
`bounce` probes with exactly that (`:10949`), matching the source's temporary `normalHitbox` swap. So the
round-200 hypothesis ("the simulator uses a different rectangle for the bounce step") is **refuted**.

The numbers do give the real arithmetic, though. Writing `amount` for the source's `MoveV(fromY - Bottom)`:

| | counter in | amount | result |
| --- | ---: | ---: | --- |
| game | 0.37504 | 0.08334 | `round(0.37504 + 0.08334) = round(0.45838) = 0` pixels, counter carries **0.45838** |
| simulator | 0.37504 | 0.08334 | `(0.08334) as i32 = 0` pixels, counter observed **0.0** |

The game's column reproduces the trace exactly, which confirms the mechanisms involved (`MoveV` goes through
`movementCounter`; a fractional amount can round to zero whole pixels). But the simulator's `as i32` cast
yields `0`, so its whole-pixel loop does not execute at all - which means the clearing at `:10952` cannot have
fired either. Its counter being `0.0` therefore came from **some other write** to `movement_remainder.y` on
that frame, and that write is what to find next: candidates are the physics move
(`move_axis_amount`/`move_axis_amount_inner`'s `collided` branch, which clears the counter), a state-entry
helper, or the bounce caller.

Separately worth fixing on its own: the `as i32` truncation is not what the source's `MoveV` does - the source
adds the amount to the counter and moves `round(counter + amount)` whole pixels, so a fractional amount must
still be accumulated rather than dropped. That is a modelling difference even where it does not (yet) show in
the traces.

### ROOT CAUSE for the +-280 dead ends: no vanilla map ever produces `EntityKind::Puffer` - it is named `eyebomb`

The `puffer` workstream measured this, and it invalidates the last two rounds of +-280 reasoning:

- vanilla Celeste's Puffer is the map entity named **`eyebomb`** (`Level.cs:677-679` -> `Add(new Puffer(...))`),
  but `map.rs:1647` maps only the literal name `"puffer"` (plus the playground fixture);
- byte-searching all 27 `.bin` files under `vendor/celeste-game/Content/Maps` for `puffer`/`pufferFish` hits
  only `Sprites.xml`, never a map;
- all 19 `eyebomb` rooms are Farewell (`LostLevels` `b-*`, `e-*`, `j-*`, `end-golden`) and every one is
  visited by the traces;
- `decode_probe room LostLevels.bin j-07` prints `name="eyebomb" kind=Unknown` for all 11 puffers and zero
  `kind=Puffer`.

So the simulator's Puffer branch - including the unconditional `explode_launch` at the old
`sim.rs:10725-10728` that this log described as "explodes a Puffer when the game refuses" - **has never
executed on the corpus**. That claim described unreachable code, and the +-280 cluster cannot be the Puffer.
Two consequences:

1. The Puffer gate landed inert (`0 / 1468 / 0`, `0 / 918 / 0`, `0 / 20 / 0`) for a reason deeper than "the
   corpus does not exercise it": the entity kind is never constructed at all. The work is still faithful and
   unit-tested (357 lib tests on its branch) and harmless for `"puffer"`-named maps, but it moves nothing here.
2. The right fix is an **entity mapping**: map `eyebomb` -> `EntityKind::Puffer`, then model the `Idle` sine,
   the `Hit` drift, the `Gone` return curve and the `PufferCollider` order. All 19 Farewell rooms stand to
   move at once, which is a far better target than the gate.

Also corrected: a fresh level load constructs a Puffer with `cantExplodeTimer = 0` - the constructor never
assigns it, only `GotoIdle`'s `Gone` branch arms the 0.5 s - so "0.5 s on spawn" is true only for a respawn.

This is the fourth time in this session that a plausible mechanism was retired by measurement rather than
landed (phantom solid, Seeker, Puffer gate, and now the Puffer entity name). The pattern to keep is the one the
workstream used: count the entities and search the assets before believing a signature.

### State after the eyebomb mapping: master already has a gated Puffer; the residual is `gone_timer`/`GotoIdle`

The `"eyebomb" => EntityKind::Puffer` mapping (`map.rs`) landed on its own and is **inert** on all three traces
(`0 / 1468 / 0`, `0 / 918 / 0`, `0 / 20 / 0`, totals unchanged) - so the entity is now constructed from vanilla
rooms and the 19 Farewell rooms' first divergences all sit before any Puffer interaction. `master` is `72b9e48`.

Combined with what is already on master this closes the Puffer question for now:

- the gate landed earlier (`27a6649`): `PufferSnapshot { state, cant_explode_timer, center }`, `initialize_puffers`,
  `advance_puffers`, and the launch gated on `state != Gone && cant_explode_timer <= 0.0` - so the run measured
  above is the **mapping + gate** combination, and it is neutral;
- the `puffer` workstream's branch (based on `577a0de`, before that landing) is therefore mostly redundant, but
  its extras are real: `gone_timer` with `GotoIdle` re-arming the 0.5 s, the `PUFFER_*` constants, and a unit test
  `puffer_explode_gate_follows_the_source_state_and_timers` (357 tests there vs 356 here);
- the residual gap in the landed version is that `advance_puffers` has no `gone_timer`/`GotoIdle`, so a Puffer
  that explodes stays `Gone` forever instead of respawning after 2.5 s; and the **stomp** branch is still
  ungated even though `Puffer.cs:552` blocks it too.

Cheapest path to close both: cherry-pick the branch's code hunks (`sim.rs`, `types.rs`, `lib.rs`, nothing in
`tas_fidelity.rs` beyond a field-coverage entry) - its own report says they do not collide with master and that
only `docs/tas-replay.md` conflicts. That is a small, self-contained follow-up rather than a new investigation.

### ROOT CAUSE for the g-01 tail: the bounce helpers hard-set the position instead of moving by counter

`sim.rs`:

```rust
10854: fn super_bounce(p: &mut PlayerSnapshot, from_y: f32) {
10857:     p.pos.y = from_y;                        // hard set
10858:     p.movement_remainder.y = 0.0;            // and the counter is thrown away
...
10865: fn side_bounce(p: &mut PlayerSnapshot, dir: i8, spring: Rect) {
10869:     p.pos.y += (from_y - p.pos.y).clamp(-4.0, 4.0);
10875:     p.movement_remainder = Vec2::default();  // both axes dropped
```

while the source (`Player.cs:2717`, `:2749`) does:

```csharp
base.Collider = normalHitbox;
MoveV(fromY - base.Bottom);                            // counter-based whole-pixel move
...
MoveV(Calc.Clamp(fromY - base.Bottom, -4f, 4f));        // SideBounce
MoveH(fromX - base.Left);                               // and a horizontal one
```

This matches the g-01 measurement exactly: at offset 1196 both sides had the same position (the bounce moved
zero whole pixels on both) but the game carried `movement_remainder.y = 0.45838` out of the frame while the
simulator had `0.0` - because the simulator discarded the fraction (`as i32` truncation in `bounce`, which is
now fixed to accumulate, and a hard clear in `super_bounce`/`side_bounce`, which is not). The one-pixel
position difference appears on the following frame.

The fix is the same shape as the wind and the `bounce` change already landed: make these helpers move by
`movementCounter` rather than assigning the position - `move_axis_amount_silent` is the existing helper that
does exactly that (counter accumulation, whole-pixel steps, counter cleared only on a blocked step, no
`OnCollide`), and both call sites (`sim.rs:10652`/`:10655`, the Spring branch) already have `map` in scope, so
threading it into the two helpers is a two-signature change.

Note this supersedes part of the previous note: `bounce` accumulating the amount was necessary but not
sufficient - the real defect for g-01 is in `super_bounce`/`side_bounce`, which are what the Spring interaction
calls.

### spring fix attempt: both bounce bodies are patched, the call sites are not what I assumed

Attempted the `super_bounce`/`side_bounce` counter fix. Both function bodies and both signatures matched
uniquely and were rewritten (to `move_axis_amount_silent`), but the build failed with:

```
error[E0308]: mismatched types
error[E0061]: this function takes 4 arguments but 3 arguments were supplied
```

for two reasons, and the second is the informative one:

1. `super_bounce`'s call site matched, but `map` there is presumably a `&Map` (the enclosing loop borrows
   `map.entities`), so passing it to a `&mut Map` helper is a type error - `E0308`;
2. `side_bounce`'s call site did **not** match the text I guessed (the pattern count was 0), so it kept the old
   3-argument form - `E0061`.

Reverted in the same step and the tree is clean. The fix is still the right one, but the next attempt must
read the actual Spring branch first (both call sites verbatim, and how `map` is borrowed there) rather than
patching from the earlier partial reads - the loop that holds the `&Entity` is exactly where a `&mut Map` call
needs the borrow to be released, so it may need the two values copied into locals and the entity borrow ended
before the call, or the helper to be split so it does not need `&mut Map` for the vertical case.

### spring fix: the exact call sites, and the borrow constraint that decides the shape of the fix

Verbatim (`sim.rs:10649-10657`):

```rust
EntityKind::Spring if p.state != PlayerState::DreamDash => {
    if entity.direction.y < 0.0 {
        if p.speed.y >= 0.0 {
            super_bounce(p, entity.bounds.y);
        }
    } else if entity.direction.x != 0.0 {
        side_bounce(p, entity.direction.x.signum() as i8, entity.bounds);
    }
}
```

The previous attempt failed because I patched from guessed text: the `super_bounce` line happened to match,
the `side_bounce` line did not, so it kept the old arity (`E0061`). Replace by **regex on the function name**
instead - `super_bounce\(p, ([^)]*)\)` and `side_bounce\(p, ([^)]*)\)` -> `... (p, map, $1)` - which is
whitespace- and argument-agnostic.

The real constraint is the borrow. These arms live inside `match entity.kind`, and `entity` is a borrowed
element of `map.entities`, so that immutable borrow of `map` is live for the whole match - passing `&mut map`
into the helpers is a conflict (`E0308` in the failed attempt came from exactly this). The two moves only ever
need **read-only** map access (`non_dream_solid_at`, `dream_block_at`), so the fix should either:

1. add a read-only sibling of `move_axis_amount_inner` that takes `&Map` (the `&mut` is only needed for the
   dream-dash branch, which a spring bounce never triggers), and have `super_bounce`/`side_bounce` call that;
   or
2. inline the counter arithmetic plus the whole-pixel probe loop inside the two helpers, which is what `bounce`
   already does (it takes `&Map` precisely because of this).

Option 2 is the smaller change and matches the existing `bounce` signature, so it is the one to take.

### LANDED: spring bounces move by movementCounter (master `1529e6b`) - +39 segments, +11,246 frames on 202

`super_bounce`/`side_bounce` assigned the position and cleared the counter where the source moves through
`Actor.MoveV`/`MoveH` (`Player.cs:2717`, `:2749`) after swapping in `normalHitbox`. Replaced with a read-only
`spring_move(p, map: &Map, horizontal, amount)` helper (counter accumulation, `round_ties_even` whole-pixel
steps, counter cleared only on a blocked step, probing with `player_rect`), which is also why the helpers can
take `&Map` and be called from inside `match entity.kind` without a borrow conflict.

Verified on all three traces against the pre-fix reports, `regressed=0` everywhere, 356 tests green:

| trace | improved / identical / regressed | ok | frames | exact |
| --- | --- | ---: | ---: | ---: |
| 202 | **76 / 1392 / 0** | 520 -> **559** | 161,355 -> **172,601** | 160,387 -> **171,672** |
| 100pct | **44 / 874 / 0** | 340 -> **362** | 97,134 -> **103,154** | 96,544 -> **102,586** |
| 1a | **3 / 17 / 0** | 16 -> **18** | 2,129 -> **2,343** | 2,125 -> **2,341** |

Two process notes worth as much as the fix:

- **Attribution control.** The workstream built an unpatched binary at `d6233b5` and ran 202 with it:
  `0 / 1468 / 0` (520 / 161,355 / 160,387 exactly), so the entire gain is this patch - notably the earlier
  `bounce` counter commit (`9eceb09`) contributes **nothing** on 202 and was landed only as a faithful
  modelling fix. Any future multi-commit stretch should do this: measure the base, not just the tip.
- **The test gate caught a real problem and cost a revert.** My first landing attempt measured the same 76/1392/0
  but the two in-file unit tests still called the old signatures, so `cargo test` failed to compile and the
  script reverted the whole change - correct behaviour, and the fix was simply to update those two call sites to
  `&Map::default()`. A gate that reverts a big win is still worth having; the alternative was pushing a red tree.

Residual on the same segment: `7-Summit|0|g-01|134447` now diverges at offset 1872, where the game's and the
simulator's `movementRemainder` are **bit-identical** (`[0.26174449920654297, 0.4297199249267578]`) and the new
reason is `facing` - a different, later defect.

**Next sweep, from the same defect class:** the enumeration of `movement_remainder` writers in `sim.rs`
(`3361`, `6821`, `8421`, `10018/10020`, `10088`, `10125/10137`, `10165/10176`, `10197`, `10226`, `10244`,
`11056`, `11439`) is now a checklist. Any site that *assigns* a position or clears the remainder where the
source calls `MoveV`/`MoveH` is the same bug as this one; the +11,246-frame gain says it is worth auditing each.

### Spring-class audit, first pass: the "clear" half is clean; the "assign the position" half is a 30-site survey

The spring fix (`1529e6b`, +76 improved segments on 202) came from one instance of a class: a helper that
**assigned the player's position** where the source moves through `movementCounter` (`MoveV`/`MoveH`). Auditing
the class needs two lists, and only the first is finished.

**Cleared-counter sites (`movement_remainder` writers).** After the spring fix the list is `3361`, `6821`,
`8421`, `9952-9957` (the legitimate accumulation inside `move_axis_amount`), `10018/10020`, `10088`,
`10125/10137`, `10165/10176`, `10197`, `10226`, `10244` (all inside the `move_axis_amount_inner` collide
branches, i.e. blocked-step semantics), `10860/10862` + `10981-10990` (the new `spring_move` and `bounce`),
`11088`, `11471`. Checked the two unaudited full clears:
- `11088` in `try_begin_badeline_boost`, right after `move_to_position` - arrival semantics;
- `11471` in `update_transition`, documented against `Player.TransitionTo`, which really does
  `ZeroRemainderX/Y` + speed rounding on reaching the target (`Player.cs`, read earlier).
Both faithful, so this half has no remaining defects. Note also `7130`'s comment ("MoveV, so its fractional
displacement must share movement_remainder.y") - a deliberate modelling note, not a bug.

**Direct `p.pos` assignments (30+ sites).** These are the other half: a site that assigns the position does not
touch `movement_remainder` at all, so the previous list cannot see it. The grep
`^\s*p\.pos(\.(x|y))?\s*(=|\+=|-=)` yields roughly thirty, and the ones that look like they could be the same
defect are `3004/3012` (moving-solid carry), `5683` (assign from an entity position), `6204-6226` (static-mover
carry), `6315` (`p.pos.y -= amount`), `7385`, `7405` (`+= facing_dir`) and `8336/8337`; the `3312/3316`,
`3335`, `3974/3978`, `4001` group is a probe-and-restore pattern and looks legitimate, as do the whole-pixel
`p.pos.x += sign` loops at `4050/4052`, `4085/4087`.

Each needs a source check against `Player.cs` before it can be called a defect, which makes this a survey
rather than a fix - and the +11,246-frame payoff from the first instance says it is worth doing properly.

### g-01 offset 1872: the divergence is `facing` alone, on the frame a dash freeze ends

Fresh dump of `7-Summit|0|g-01|134447` (now replaying to 1873 frames after the spring fix):

```
offset=1869 row=136357 StDash pos=(27410,-19548) counter=(0.43331,0.25815) move=(0,0) speed=(0,0) stalled=true freeze=0.05000
offset=1870 row=136358 StDash ... identical ... stalled=true freeze=0.03333
offset=1871 row=136359 StDash ... identical ... stalled=true freeze=0.01667
offset=1872 row=136360 StDash pos=(27413,-19551) counter=(0.26174,0.42972) move=(2.82843,-2.82843) speed=(169.70563,-169.70563) game==rust freeze=0.00000
```

At offset 1872 **every field the dump prints agrees** - position, both counters, both move components, both
speed components, the state and the freeze timer - and the harness still records a divergence, whose reason is
`facing`. So the only difference is in a field the dump does not carry: the player's facing direction, on the
first un-frozen frame after the three-frame dash freeze (`freeze` counting 0.05 -> 0.0333 -> 0.0167, which is
`DASH_FREEZE_TIME`).

That is a much narrower lead than the previous ones: the state is `StDash`, the dash's freeze accounting already
matches, and the velocity produced on the exit frame matches to the printed precision - only the direction the
player is turned differs. The next step is to compare the `facing` writes on both sides for the dash path
(`Player.DashUpdate`/`DashBegin` versus the simulator's `dash_update`/`dash_begin`), looking for an assignment
that happens one frame earlier or later, rather than for a missing write.

Also worth noting for future dumps: this case shows the dump's field set is the limiting factor when the
divergence is in an unprinted field, and the harness's `reasons` list is what identifies it.

### g-01 offset 1872 (`facing`): the two corresponding writes are `Player.cs:4493` and `sim.rs:8486`

| source | simulator |
| --- | --- |
| `Facing = (Facings)Math.Sign(DashDir.X);` (`Player.cs:4493`) | `p.facing = p.dash_dir.x > 0.0;` (`sim.rs:8486`) |

Both derive the facing from the dash direction, so this is a timing question rather than a missing write:
`Player.cs:4493` sits late in the file, which points at `DashUpdate`/the dash coroutine rather than `DashBegin`,
i.e. the write may happen on a different frame on the two sides.

One corner to keep in mind while comparing: the source's `(Facings)Math.Sign(DashDir.X)` produces the enum
value `0` when `DashDir.X == 0`, whereas `p.dash_dir.x > 0.0` yields `false` (Left) in that case. If the trace
exports facing as "is Right" the two agree, so this is probably not the diverging case - and it is not this
frame's case anyway: the exit frame's move is `(2.82843,-2.82843)`, i.e. a diagonal dash with `DashDir.X = 1`.

The dash state on both sides had already matched through three freeze frames (0.05 / 0.0333 / 0.0167), and the
exit frame's position, counters, move, speeds and state all matched to the printed precision, so the remaining
work is to read `Player.DashUpdate`/the dash coroutine against the simulator's `dash_update` and find where the
facing assignment lands a frame early or late.

### The `facing` divergence: the source writes it in `DashBegin`, the simulator only in the deferred red-dash path

`Player.cs:4478-4500` is the body of `DashBegin` (it assigns `Speed`, then `gliderBoostDir = (DashDir =
value)`, then):

```csharp
4491: if (DashDir.X != 0f)
4493:     Facing = (Facings)Math.Sign(DashDir.X);
```

On the simulator side the only dash-related facing write found by grepping `p.facing =` is
`sim.rs:8486`, inside `red_dash_update`'s **deferred** block (`if p.dash_dir == Vec2::default()`, after
`state_timer` expires, where it re-derives `dash_dir` from `last_aim`). `begin_dash` does not appear to write
facing at all. So the dash's facing is set a frame late (or not by the same trigger), which matches the
measured divergence exactly: at `7-Summit|0|g-01|134447` offset 1872 every printed field agrees and only
`facing` differs, on the first frame after the dash freeze, with a diagonal dash (`DashDir.X = 1`).

Fix shape: in `begin_dash`, after `p.dash_dir` is set, mirror `:4491-4493` - `if p.dash_dir.x != 0.0 {
p.facing = p.dash_dir.x > 0.0; }` - and check whether the deferred write in `red_dash_update` is then
redundant (the source's `:4493` also guards on `DashDir.X != 0`, so keeping both is harmless if the values
agree).

### Five residual gaps handed over by the spring workstream

Its branch replicated the landed fix byte-for-byte behaviourally (segment-for-segment equal on all three traces)
and listed these, all pre-existing:

1. `Player.SideBounce`'s early-out `if (Math.Abs(Speed.X) > 240f && Math.Sign(Speed.X) == dir) return false;`
   (`Player.cs:2743-2746`) and `Spring.OnCollide`'s use of that bool to gate `BounceAnimate`
   (`Spring.cs:140-152`) are not modelled - `side_bounce` always applies the bounce. **Cheapest of the five**:
   it is a missing guard in the function just fixed.
2. `MoveVExact`'s downward `CollideFirstOutside<JumpThru>` probe (`Actor.cs:264-284`) is absent from
   `spring_move` (and from the pre-existing `bounce` helper); reachable only when the correction steps down a
   whole pixel.
3. `gliderBoostTimer = 0f` (`Player.cs:2729`, `:2769`) has no snapshot field.
4. A spring hit while in `StarFly` reaches `reset_for_spring_bounce`, which sets state Normal **without**
   running `StarFlyEnd`, unlike `bounce` (`sim.rs:10973`). Also cheap.
5. The `facing` residual above.

Also confirmed by an attribution control on its side: an unpatched binary at `d6233b5` reproduces the old
totals exactly (`0` delta), so the entire +76/+44/+3 improvement is the spring patch.

### Correction: the facing write belongs to the dash *coroutine*, not `DashBegin` - and the target is `dash_update`'s publish block

My previous note placed the source's facing write (`Player.cs:4491-4493`) in `DashBegin`. Reading
`begin_dash` (`sim.rs:7656-7688`) shows that is wrong, and the simulator's own comment says why:

```rust
7665: // DashBegin clears DashDir. DashCoroutine does not sample lastAim until it
7666: // resumes after its initial yield (and any Celeste.Freeze frames).
7667: p.dash_dir = Vec2::default();
```

So `DashBegin` **clears** `DashDir`; the source lines 4478-4500 - which assign `Speed`, then
`gliderBoostDir = (DashDir = value)` and then `Facing = Math.Sign(DashDir.X)` under `if (DashDir.X != 0f)` -
are the body of **`DashCoroutine` after its initial yield**, which is also why a dash freeze (three frames of
`DASH_FREEZE_TIME`) sits between the clear and the write. That matches the measurement: the `facing`
divergence appears on the first un-frozen frame.

An attempt to add the write to `begin_dash` was a **measured no-op** (g-01 slice unchanged) because `dash_dir`
is zero there, and it was reverted - correctly, since the guard `dash_dir.x != 0.0` can never pass at that
point.

The correct target is the **publish block in `dash_update`** - the same place already studied for the
`dash_dir = last_aim` / `speed = (dir * DASH_SPEED)` publish, which mirrors the coroutine's resume. The red-dash
path already has the write (`red_dash_update:8486`, `if p.dash_dir.x != 0.0 { p.facing = p.dash_dir.x > 0.0; }`),
which is the pattern to copy; the ordinary dash path does not.

So: add `if p.dash_dir.x != 0.0 { p.facing = p.dash_dir.x > 0.0; }` immediately after the publish's
`p.dash_dir = p.last_aim;` in `dash_update`, then verify with `--rooms g-01` (expect the `134447` segment to
pass offset 1872).

### LANDED: the dash facing publish (master `a967db1`) - +9 ok segments and +3,374 frames on 202

`dash_update`'s publish block set `dash_dir` and `speed` from the aim but never published the **facing**, which
`Player.DashCoroutine` does immediately afterwards (`Player.cs:4491-4493`, `if (DashDir.X != 0f) Facing =
(Facings)Math.Sign(DashDir.X);`). The red-dash path already had that write (`red_dash_update`), the ordinary
dash path did not, so facing landed a frame late - exactly the residual measured at `7-Summit|0|g-01|134447`
offset 1872, where every other printed field agreed.

Verified against the post-spring baselines, `regressed=0`, 356 tests green:

| trace | improved / identical / regressed | ok | frames | exact |
| --- | --- | ---: | ---: | ---: |
| 202 | 22 / 1446 / 0 | 559 -> **568** | 172,601 -> **175,975** | 171,672 -> **175,055** |
| 100pct | 12 / 906 / 0 | 362 -> **366** | 103,154 -> **105,167** | 102,586 -> **104,603** |
| 1a | 0 (vs the post-spring baseline) | 18 | 2,343 | 2,341 |

Note on the 1a row in the landing output: it was diffed against `gate-tg2-1a.json` (the **pre-spring** 1a
baseline) rather than `spr-1a.json`, so its `improved=3` double-counts the spring patch. The post-fix totals
(18 / 2,343 / 2,341) are identical to the post-spring ones, i.e. this change is neutral on 1a. Baselines must
move with every landing - this log now uses `spr-*` for spring and `face-*` for facing.

Two attempts were needed and the first was a measured no-op worth remembering: adding the write to
`begin_dash` changed nothing, because `DashBegin` clears `DashDir` (the simulator's own comment says so,
`sim.rs:7665-7667`), so the guard could never pass. The coroutine-resume site (the publish block) is the right
one, and the g-01 slice moved immediately: `134447` 1873 -> **1967** and `356101` 1466 -> **1560**, +94 frames
each. The next divergence on `134447` is now offset 1966.

### LANDED: `SideBounce` refuses a fast same-direction player (master `afd1288`) - +2 ok, +1,703 frames

`Player.SideBounce` has an early-out (`Player.cs:2743-2746`): `if (Math.Abs(Speed.X) > 240f &&
Math.Sign(Speed.X) == dir) return false;` - a spring will not bounce a player already moving that way quickly
(and `Spring.OnCollide` uses that `false` to skip `BounceAnimate`, `Spring.cs:140-152`, which is cosmetic and
still unmodelled). The simulator always applied the bounce. Landing the guard as an early `return` in
`side_bounce` is enough for the observable behaviour.

Measured against the post-facing baselines (`face-*`), `regressed=0`, 356 tests green:

| trace | improved / identical / regressed | ok | frames | exact |
| --- | --- | ---: | ---: | ---: |
| 202 | 7 / 1461 / 0 | 568 -> **570** | 175,975 -> **177,678** | 175,055 -> **176,760** |
| 100pct | 3 / 915 / 0 | 366 -> **367** | 105,167 -> **105,958** | 104,603 -> **105,395** |
| 1a | 0 / 20 / 0 | 18 | 2,343 | 2,341 |

Worth noting for future slices: the `--rooms g-01` slice showed **zero** change, so this looked inert until the
full traces ran - the guard fires in other rooms. A slice proves a change *can* matter; it never proves it is
inert. (The reverse also holds, and both directions have now been seen in this log: the round-197 `eyebomb`
mapping was neutral on whole traces, and this one was neutral on the slice but not on the traces.)

This stretch's cadence, all from the one defect class found with the spring fix: spring counter move (+39 ok),
dash facing publish (+9 ok), `SideBounce` early-out (+2 ok) - 202 went from 520 to **570** ok and 161,355 to
**177,678** frames.

### Incident: my dash-facing commit stored `sim.rs` as CRLF and would have blocked every workstream merge

`a967db1` (the dash facing publish) committed `crates/celeste-physics/src/sim.rs` with **CRLF** line endings -
the moveclass workstream measured the blob as `crlf=23824, lf=2` where every earlier commit had `crlf=0`. Cause:
my own tooling. The patch pattern used all stretch long is `Set-Content -Path $src -Value $s -Encoding utf8
-NoNewline` after string surgery with backtick-`r`-backtick-`n`, and that rewrites the whole file with CRLF in
the working tree; with `core.autocrlf=true` the staging step then stored CRLF in the blob. Any branch based on an
older LF commit consequently conflicts over the entire file - the workstream hit a single 47,729-line conflict
region even with `merge.renormalize=true`.

Fixed on master in `ecbc03a`: added `crates/celeste-physics/src/sim.rs text eol=lf` to `.gitattributes` (next to
the pre-existing `fixtures/e2e/*.json` rule) and `git add --renormalize`d the file. Verified by counting bytes of
the committed blob (`git cat-file blob <rev>:<path>` then `[IO.File]::ReadAllBytes`): **cr=2, lf=23836** - two
stray CRLF lines remain at the `p.dash_dir = p.last_aim;` sites, which is a two-line diff, not a whole-file one.

Two lessons, both already foreshadowed in this log:

1. **Do not verify line endings through a PowerShell pipeline.** `git show HEAD:path | Out-String` converted
   LF to CRLF in my first check and made the fixed blob look broken; only the byte count is trustworthy.
2. **The patch helper itself is the hazard.** Every `Set-Content` rewrite risks re-encoding the file; the
   `.gitattributes` rule now normalizes on staging, which makes that hazard harmless for this path - but the
   same pattern applied to any other source file would repeat the incident there, so the attributes should be
   widened (or the edits done with a tool that does not rewrite the whole file).

### StarFly spring end: verified inert, and not in master (recorded, not landed)

The two-step edit (thread `map` into `reset_for_spring_bounce`, then run `end_star_fly` before assigning
`PlayerState::Normal`, matching `bounce` at `sim.rs:10964`) built and passed the 356 tests, and the full traces
measured `0 / 1468 / 0`, `0 / 918 / 0`, `0 / 20 / 0` against the current `side-*` baselines - i.e. **inert** on
all three. The landing job then reported "nothing added to commit" and "Everything up-to-date", so the edit had
already been reverted out of the working tree; it is **not** in master.

Recording it rather than re-applying: the change is inert on the corpus (no trace hits a spring while in
`StarFly`), it is faithful but low-value, and re-measuring it costs three full traces. The recipe is above if a
future corpus needs it - and the same "inert-but-correct" judgement already covers several landed items
(`Session.DoNotLoad`, the Puffer gate, the `bounce` counter), so the difference here is only that this one was
not worth the second measurement.

**What to do instead, from the moveclass survey's out-of-class leads - these are missing mechanics with real
win potential, not stylistic fixes:**

1. **`DuckCorrectCheck` / `DuckCorrectSlide`** (`Player.cs:2840-2855`): inside `NormalUpdate`'s duck block, when
   `Speed.X == 0` the source runs `MoveH(+-50f * Engine.DeltaTime)` - a **counter-based move** that creeps a
   ducked player sideways. `sim.rs`'s duck block (~7423-7431) only has the `CanUnDuck -> Ducking = false`
   branch, so the crawl is **not modelled at all**. Same family as the spring fix, and a missing mechanic rather
   than a wrong one.
2. **`Player.FlingBirdUpdate`** (`StFlingBird`, state 24) - no implementation at all.
3. **`Player.MoonLanding`**'s `MoveV(-200f * Engine.DeltaTime)` - unimplemented.

### The `DuckCorrect*` lead is suspect: the constants are dead in the vendored source

The moveclass survey listed `DuckCorrectCheck`/`DuckCorrectSlide` as an unmodelled mechanic, citing
`docs/Player.cs:2840-2855`. Two checks say otherwise:

- in the **vendored** source (`vendor/celeste-fna/Celeste/Player.cs`) those names appear **only** at their
  declarations - `private const int DuckCorrectCheck = 4;` (`:151`) and `private const float DuckCorrectSlide =
  50f;` (`:153`) - with **no use site anywhere in the file**. They are dead constants there;
- the simulator's duck block (`sim.rs:7423-7431`) does contain only the enter/exit logic, which matches a game
  that never nudges: `if p.ducking { if on_ground && move_y != 1 && can_unduck { ducking = false } } else if
  on_ground && move_y == 1 && speed.y >= 0.0 { ... }`.

So the survey's citation points at `docs/Player.cs`, which is a **different copy of the source** - not the
authority for what the traced game ran. Implementing a slide the traced build does not have would *introduce* a
divergence rather than remove one.

Rule to apply to the other two leads before any implementation: `Player.FlingBirdUpdate` (`StFlingBird`, state
24) and `Player.MoonLanding`'s `MoveV(-200f * Engine.DeltaTime)` must be located **in
`vendor/celeste-fna/Celeste/Player.cs`** and shown to be reachable from the traced states. `docs/Player.cs` is
useful as a cross-reference but cannot be the basis for a change, and this log has now twice been misled by a
reference to a copy rather than the source of record (the other being the round-192 line numbers).

### Lead verification against the source of record: `FlingBird` is real, `MoonLanding` is not

Applying the rule from the previous note to the survey's remaining two leads, in
`vendor/celeste-fna/Celeste/Player.cs`:

- **`FlingBird` is legitimate.** `StFlingBird = 24` (`:397`), the state is registered at `:1170`
  (`StateMachine.SetCallbacks(24, FlingBirdUpdate, FlingBirdCoroutine, FlingBirdBegin, FlingBirdEnd)`),
  `DoFlingBird` at `:5541`, `FinishFlingBird` at `:5556`, and the state functions at `:5568-5578`. The
  simulator has only the enum variant (`types.rs:46`, `FlingBird = 24`) and no implementation, so this is a
  genuinely missing mechanic rather than a copy artefact.
- **`MoonLanding` does not exist** in the source of record - no matches at all - so that lead is dead, like
  `DuckCorrectCheck`/`DuckCorrectSlide`.

Before implementing `FlingBird`, one more cheap check decides whether it is worth anything: whether any row the
202 trace actually visits is in `StFlingBird` (state 24). The trace exports the state name per player row, and
the class worklist's anchor states are read from those rows - `StFlingBird` appears in none of the top classes,
which is suggestive but not conclusive. If the state never occurs in the corpus, implementing it would be a
third inert-but-correct item in a row, and the better target is the measured divergence at
`7-Summit|0|g-01|134447` offset 1966, which comes from the trace itself and depends on no copy of the source.

### Corpus state distribution, and the `FlingBird` verdict: 635 rows, real and unmodelled

A one-script scan of `trace-202-v7.jsonl` (counted straight off the `"state"` field, 22 distinct states) gives
the frequency table to prioritise state work by:

| state | rows | | state | rows |
| --- | ---: | --- | --- | ---: |
| `StNormal` | 250,547 | | `StSummitLaunch` | 4,248 |
| `StDash` | 116,203 | | `StIntroJump` | 3,899 |
| `StDummy` | 16,241 | | `StIntroWakeUp` | 2,151 |
| `StStarFly` | 10,460 | | `StIntroWalk` | 1,864 |
| `StLaunch` | 8,311 | | `StPickup` | 1,830 |
| `StDreamDash` | 6,950 | | `StAttract` | 1,578 |
| `StRedDash` | 6,058 | | `StIntroRespawn` | 920 |
| `StClimb` | 4,454 | | **`StFlingBird`** | **635** |
| | | | `StCassetteFly` | 562 |
| | | | `StTempleFall` | 414 |
| | | | `StBoost` | 304 |
| | | | `StSwim` | 250 |
| | | | `StIntroThinkForABit` | 97 |
| | | | `StHitSquash` | 25 |

So the survey's third lead is the good one: `StFlingBird` occurs 635 times in the 202 trace, the state is
registered in the source of record (`Player.cs:1170`), and the simulator has only the enum variant
(`types.rs:46`) - those rows are certainly simulated wrong today. Unlike `DuckCorrect*` and `MoonLanding`, this
one is both real and exercised, so implementing it can actually move segments.

Also worth using the table for: `StCassetteFly` (562), `StTempleFall` (414), `StBoost` (304) and `StSwim` (250)
are small but non-zero, and `StHitSquash` (25) is the kind of state that would be invisible in aggregate
metrics but decisive for a handful of segments.

### `StFlingBird` (state 24): a small, fully-specified mechanic the simulator does not have

All of it is four functions in the source of record (`Player.cs:5541-5588`):

```csharp
public bool DoFlingBird(FlingBird bird) {          // called by the FlingBird entity
    if (!Dead && StateMachine.State != 24) {
        flingBird = bird;  StateMachine.State = 24;
        if (Holding != null) Drop();
        return true;
    }
    return false;
}
public void FinishFlingBird() {                    // called when the flight ends
    StateMachine.State = 0;  AutoJump = true;
    forceMoveX = 1;  forceMoveXTimer = 0.2f;
    Speed = FlingBird.FlingSpeed;  varJumpTimer = 0.2f;  varJumpSpeed = Speed.Y;  launched = true;
}
private void FlingBirdBegin() { RefillDash(); RefillStamina(); }
private void FlingBirdEnd() { }
private int FlingBirdUpdate() {
    MoveTowardsX(flingBird.X, 250f * Engine.DeltaTime);
    MoveTowardsY(flingBird.Y + 8f + base.Collider.Height, 250f * Engine.DeltaTime);
    return 24;
}
private IEnumerator FlingBirdCoroutine() { yield break; }
```

So `StFlingBird` is "carried by the bird": each frame the player is moved toward the bird at 250 px/s with
`MoveTowards` - the **same counter-based `MoveH`/`MoveV` family** as the wind and the spring fixes, so the
movement itself is the already-solved shape. `FlingBirdBegin` refills dash and stamina, and `FinishFlingBird`
hands control back with `Speed = FlingBird.FlingSpeed`, auto-jump and a 0.2 s force-move to the right.

The simulator has the enum variant (`types.rs:46`) and **nothing else** - a grep for `fling` in `sim.rs`
returns nothing. That also explains the 635 `StFlingBird` rows in the 202 trace: the trace records the *game's*
state on those rows while the simulator is in whatever state it thinks it is, so the row comparison fails there
by construction.

Implementation checklist before writing code: (a) is the `FlingBird` **entity** decoded at all (an `EntityKind`
variant and a map-name mapping - the same check that caught `eyebomb`)? (b) what does the entity call, and when
(`DoFlingBird`/`FinishFlingBird` are invoked from the entity's own update, which may itself be unmodelled);
(c) does anything in the simulator ever set state 24 today? If the entity is missing, this grows from a small
`MoveTowards` change into a new-entity piece, and the frequency table says it is worth 635 rows of the corpus.

### `StFlingBird` verdict: the entity is missing too, and the work is bounded to the Farewell bird section

The three checks from the previous note, all read-only:

1. **No entity kind.** `FlingBird` appears in `types.rs` only as the *player state* enum variant
   (`:46`, `FlingBird = 24`). There is no `EntityKind::FlingBird`, and `map.rs` has no name mapping for it, so
   the entity is not decoded at all - the "new entity" case, not the "small state" case.
2. **No simulator references.** A grep for `FlingBird` in `sim.rs` returns nothing: nothing reads the state,
   nothing sets it.
3. **Bounded blast radius.** Every one of the 635 `StFlingBird` rows is in `Celeste/LostLevels`, across nine
   rooms - `j-03` 127, `j-06` 97, `j-05` 96, `j-02` 95, `j-01` 63, `j-08` 63, `j-04` 33, `j-10` 31, `j-07` 30.
   That is the Farewell bird sequence and nothing else.

So this is a new-entity piece rather than a one-function state: decode the entity, model when it calls
`DoFlingBird`/`FinishFlingBird`, and add the carried-by-bird state whose movement is `MoveTowards(bird, 250 *
dt)` - the already-solved counter-move shape - plus `FlingBirdBegin`'s dash/stamina refill and
`FinishFlingBird`'s hand-back (`Speed = FlingBird.FlingSpeed`, auto-jump, 0.2 s force-move right).

Cost estimate and comparison: one new entity plus one state, affecting nine rooms; by the corpus frequency
table that is 635 rows, against `StCassetteFly` 562, `StTempleFall` 414, `StBoost` 304 and `StSwim` 250 - all
of which are *smaller* and may each be a state the simulator already half-models. Worth doing, but it should be
scheduled as a piece of work rather than squeezed into a single step, and the measured divergence at
`7-Summit|0|g-01|134447` offset 1966 remains the cheaper target for the next step.

### g-01 offset 1966: a dash where the game hits a ceiling and the simulator corners around it by 4 px

```
offset=1965  game == rust: pos=(27328,-19637) counter=(0.12650,0.48859) move=(-2.82843,-2.82843) speed=(-169.70563,-169.70563)
offset=1966  game: pos=(27325,-19637) counter=(0.29807, 0.00000) move=(-2.82843,-0.48859) speed=(-169.70563,  0.00000)
             rust: pos=(27321,-19638) counter=(0.29807, 0.00000) move=(-6.82843,-1.48859) speed=(-169.70563,-169.70563)
```

Three readings:

1. The simulator's x move is exactly **4.0 px larger** than the game's (`-6.82843 = -2.82843 - 4.0`), and 4 is
   the range of Celeste's dash **corner correction** (the `for (int i = 1; i <= 4; i++)` `MoveHExact` loops noted
   in an earlier round).
2. The game's `speed.y` becomes **0** while the simulator's stays `-169.70563`: the game hit a **ceiling** and
   took the collide response. Note this is the opposite of the wind case - the dash's own movement passes a
   callback, so `OnCollideV` **does** run here.
3. The consequences are a position difference in both axes and a speed difference in y, on the same frame.

So the two sides disagree about whether that step was blocked: the game stopped vertically and stayed put in x,
the simulator corrected around something by four pixels and carried on. The next step is to find the simulator's
corner-correction site (grep `corner`/`correct` in `sim.rs`) and compare its conditions against the source's
dash correction - in particular whether it probes with the right collider and whether it is allowed at all when
the vertical move is blocked, which is the case that looks wrong here.

### Corner correction: the simulator has one constant, the source has three (one of them 5 for upward dashes)

| simulator | source of record |
| --- | --- |
| `const DASH_CORNER_CORRECTION: i32 = 4;` (`sim.rs:55`) | `DashCornerCorrection = 4` (`Player.cs:225`) |
| - | `UpwardCornerCorrection = 4` (`:171`) |
| - | **`DashingUpwardCornerCorrection = 5`** (`:173`) |
| one loop, `for correction in 1..=DASH_CORNER_CORRECTION` (`sim.rs:10071`) | three `for (int i = 1; i <= 4; i++)` loops (`:3187`, `:3300`, `:3455`), and `DashingUpwardCornerCorrection` is used somewhere else again |

The measured divergence at `7-Summit|0|g-01|134447` offset 1966 is an **upward diagonal dash** (speed
`(-169.70563,-169.70563)`) where the game's vertical speed was zeroed by a ceiling hit and its x move stayed at
`-2.82843`, while the simulator moved `-6.82843` - exactly 4 px of horizontal correction - and kept its vertical
speed. A single 4-px constant applied to every dash state is a plausible cause: the source distinguishes the
dash's direction, and for a dash going up it uses a **5-px** range in its own branch.

Next: read the three source loops (`:3187`, `:3300`, `:3455`) and wherever `DashingUpwardCornerCorrection` is
consumed, and map each to the conditions that select it; then compare with `sim.rs:10056-10090` to see which
state/direction combinations the single loop is being applied to that it should not be.

### Correction: the upward constants are dead too; the g-01 offset-1966 difference is WHEN `speed.y` becomes zero

Two findings, one of which retires the previous note's hypothesis.

**1. `DashingUpwardCornerCorrection = 5` and `UpwardCornerCorrection = 4` have no consumers** in the source of
record - like `DuckCorrectCheck`/`DuckCorrectSlide`, they appear only at their declarations (`Player.cs:171`,
`:173`). So "the source uses 5 px for an upward dash" is **withdrawn**; the traced build does not appear to use
those constants at all. That is the third time in this log that a lead sourced from a copy of the source rather
than `vendor/celeste-fna/Celeste/Player.cs` had to be retired.

**2. The simulator's horizontal correction is gated on `speed.y == 0`.** `sim.rs:10067-10088`:

```rust
if matches!(p.state, PlayerState::Dash | PlayerState::RedDash)
    && p.speed.y == 0.0
    && p.speed.x != 0.0
{
    for correction in 1..=DASH_CORNER_CORRECTION { ... p.pos.y += offset; p.pos.x += sign as f32; return; }
}
```

and the measured divergence matches that loop exactly: the simulator's x move is 4.0 px larger while the game's
`speed.y` is `0.0` and the simulator's is still `-169.70563`. So the correction probably fired in the simulator
because `speed.y` was still non-zero at the point the move was attempted, or because it was zero at that point in
the game but not in the simulator - i.e. the two sides disagree about **when** the vertical speed reaches zero
within the frame, not about the correction's range. The game's zero is the ceiling response; the simulator keeps
dashing upward.

Next: trace the order inside the frame on both sides - where the game's `OnCollideV` sets `Speed.Y = 0`
(`Player.cs`, the dash's vertical move) and where the simulator sets `speed.y` during the dash - and check
whether the simulator's dash move can run with a stale non-zero `speed.y`. A probe on that frame that prints
`speed.y` before and after the horizontal move would settle it in one run.

### g-01 offset 1966: not the corner correction - the simulator blocks a horizontal pixel the game does not

A frame-internal probe (`speed.y` printed on entry to the horizontal collide branch and after the correction
block) ran on the `g-01` slice and produced exactly two lines, both:

```
SY beforeH pos=(27325.00,-19637.00) sy=-169.70563 sx=-169.70563
```

Readings:

1. `(27325,-19637)` is precisely the position the dump attributes to **both** sides at offset 1966, so the probe
   landed on the divergence frame.
2. On entry the simulator's `speed.y` is still `-169.70563`, so the correction loop's guard (`speed.y == 0.0`,
   `sim.rs:10068`) is **false** and the four-pixel correction never runs there. The previous note's "the
   correction fired when it should not" is therefore **withdrawn** - the second retreat of a corner-correction
   hypothesis in two rounds.
3. `afterH` never printed, so the code returned before reaching it - i.e. through `try_dash_collide`'s
   `Rebound`/`Ignore` early returns (`sim.rs:10060-10064`). Meanwhile the game's `speed.x` is unchanged
   (`-169.70563`), so the game's horizontal move was *not* blocked.

So the two sides disagree about **geometry**, at the same position: the simulator's horizontal pixel probe hits a
solid the game's does not, and it happens on the first pixel (both lines are identical, and the horizontal move
runs per pixel). The next step is to compare the probe rectangle and the solids at `(27325,-19637)` on both sides
for that frame - the same kind of check that found the wind's ceiling (`current_player_rect` vs the game's
collider), and the same discipline applies: print the rect and the overlapping solids rather than reasoning about
which collider "should" be in play.

### Anchoring lesson (third time): match structure, not remembered prose

Two probe insertions failed in a row on the `g-01` geometry check, both for the same reason: I wrote the anchor
from memory instead of from the file. The first attempt searched for a comment line without the backticks the
file actually has; the second searched for `OnDashCollide branch runs before the`, and the only `OnDashCollide`
occurrence in `sim.rs` is inside the doc comment of `first_solid_entity_at`
(`/// `OnDashCollide`. Callers exclude static tile solids before using this.`) - not the comment I thought I
remembered at the dash collide branch.

The rule, now demonstrated three ways in this log (stale line numbers from a copy; a comment text from memory
here; and earlier a call-site argument list from memory), is:

- anchor on a **structural** string that must exist - a constant name (`DASH_CORNER_CORRECTION`), a function
  signature, a distinctive expression - never on remembered prose, indentation, or argument text;
- when the anchor fails, **read the file and print the surrounding lines** before retrying rather than
  re-guessing; both failures this round cost a call each and neither touched the build.

The probe itself is ready and unchanged in intent: print, on the horizontal collide branch of the dash, the
player rect, the rect one pixel ahead, `non_dream_solid_at` for that rect, and any overlapping `solids` and
entities - the same "print geometry, do not reason about it" method that found the wind's ceiling. It just needs
to be inserted relative to the known-present loop marker (`for correction in 1..=DASH_CORNER_CORRECTION`) or
found by reading the branch first.

### Correction: the simulator has three corner-correction loops, not one - and where the geometry probe belongs

Two corrections and one placement insight.

**1. Three loops, not one.** The round-229 note said the simulator has a single `DASH_CORNER_CORRECTION` loop
against the source's three. That is wrong: `for correction in 1..=DASH_CORNER_CORRECTION` occurs three times in
`sim.rs` (`:10069`, `:10172`, `:10183`), and all three live inside `move_axis_amount_inner` - the same three as
the source's `Player.cs:3187`, `:3300`, `:3455`. So the "one constant versus three" observation reduces to the
sim having **one shared constant value** where the source declares three (two of which, `UpwardCornerCorrection`
and `DashingUpwardCornerCorrection`, are dead in the source of record anyway). The corner-correction hypothesis
for the offset-1966 divergence is therefore weak on both sides, and the earlier measurements already rule it
out: on that frame the simulator enters the horizontal collide branch with `speed.y = -169.70563`, so the
correction's own guard (`speed.y == 0.0`) is false.

**2. Where the probe has to go.** The geometry probe inserted this round produced no output, and the reason is
structural rather than incidental: it was placed as a sibling of the `for correction` loop, i.e. inside
`if matches!(p.state, PlayerState::Dash | PlayerState::RedDash) && p.speed.y == 0.0 && p.speed.x != 0.0`. On the
divergence frame that guard is false, so the probe could not fire - which is the same fact that rules the
correction out. The early return that skipped the previous round's `afterH` probe happens **before** that guard,
in `try_dash_collide(p, map, next, true, sign as f32)`'s `Rebound`/`Ignore` arms.

So the geometry probe belongs immediately **before `try_dash_collide`**, printing `next` (the rect being
probed), the player rect, `non_dream_solid_at(next)`, and any overlapping `solids` and entities - that is where
the two sides can be shown to disagree about the world rather than about the response to it.

### Geometry probe results: `solidNext=true` on every dash collide, and an `Unknown` entity

The probe placed immediately before `try_dash_collide` ran 33 times on the `g-01` slice, and every line has the
same shape - the rect one pixel ahead of the player is **solid**:

```
TC pos=(25452.00,-18919.00) me=(25448.0,-18930.0,8.0,11.0) next=(25449.0,-18930.0,8.0,11.0) solidNext=true ents=[CrystalStaticSpinner@(25456,-18926,16,12)] solids=[(25456,-18920,152,8)]
TC pos=(25564.00,-18967.00) me=(25560.0,-18978.0,8.0,11.0) next=(25561.0,-18978.0,8.0,11.0) solidNext=true ents=[Unknown@(25560,-18968,8,8)] solids=[(25568,-18984,136,8) (25568,-18976,216,8) (25568,-18968,656,8)]
TC pos=(26076.00,-19109.00) ... solidNext=true ents=[] solids=[(26080,-19120,16,8) (26080,-19112,16,8)]
```

Two things to take from this:

1. **An `Unknown` entity sits exactly on the probe rect** in at least one case (`(25560,-18968,8,8)`, with the
   player's `next` rect `(25561,-18978,8,11)` overlapping its lower half). Unmapped entity names have already
   cost this project once - the Puffer is named `eyebomb` in every vanilla map and was therefore never
   constructed - so an `Unknown` entity in a solid-probing path is worth naming: if the simulator treats unknown
   kinds as solid, that is a phantom solid, and this is the same signature the wind work chased.
2. The divergence frame at `(27325,-19637)` is **not** in the list, so on that frame the simulator never reached
   `try_dash_collide`. Combined with the earlier finding (`beforeH` fired, `afterH` did not), the early return
   happens between the horizontal branch's entry and this call - which is a narrower place to look than the
   collision response itself.

### The `Unknown` entity is `summitcheckpoint`, and `Unknown` is not solid - no phantom here

Decoding `7-Summit.bin` room `g-01` for the entity the probe reported at `(25560,-18968,8,8)` gives its name
immediately: **`summitcheckpoint`**, `kind=Unknown`. The room's distinct entity names are `badelineBoost`,
`cloud`, `jumpThru`, `killbox`, `player`, `refill`, `spinner`, `spring`, `strawberry`, **`summitcheckpoint`**,
`switchGate`, `touchSwitch`, `wallSpringLeft`, `wallSpringRight` - so it is the only unmapped one there.

But the solid-probing question is settled by an explicit whitelist, `is_solid_entity` (`sim.rs:10268-10288`),
which lists `BounceBlock`, `CassetteBlock`, `CrushBlock`, `DashBlock`, `DreamBlock`, `ExitBlock`,
`FallingBlock`, `InvisibleBarrier`, `MoveBlock`, `MovingSolid`, `StaticSolid`, `CrumbleBlock`,
`FloatySpaceBlock`, `ZipMover`, `TempleGate` and `DashSwitch` - and **not `Unknown`**. So an unmapped entity is
not a phantom solid, and the `solidNext=true` in the probe output came from the `solids` list (tiles), not from
the overlapping `Unknown`. The `eyebomb` lesson does not repeat here, and my probe's entity listing (which
prints *all* overlapping entities, not only solids) is what made it look otherwise.

What `summitcheckpoint` being unmapped does cost is different and not geometric: a checkpoint trigger that
never fires, i.e. session state rather than collision - the same category as the flags and cassette work, and
worth checking when a corpus segment depends on a respawn point rather than on a wall.

### The 4 px at g-01 offset 1966 does not come from the dash collide path at all

Two facts, and together they invalidate the probe pairing I had been reasoning from:

- `if horizontal {` occurs **45 times** in `sim.rs` (`move_axis:9959`, `move_axis_amount_inner:10052`, and 43
  others), so the earlier `beforeH` probe - anchored on a comment plus `if horizontal` - was not necessarily
  inside `move_axis_amount_inner`. That resolves the apparent contradiction: `beforeH` fired on the divergence
  position while `TC` did not, because the two probes were in **different functions**.
- The `try_dash_collide` call sites are only two: `10057` with `true` (horizontal) and `10111` with `false`
  (vertical). The `TC` probe was anchored on the horizontal one, so it is the right witness - and it did **not**
  fire on the divergence frame.

So the simulator never enters `move_axis_amount_inner`'s horizontal branch on that frame, and the 4 px of extra
horizontal travel must come from a different mover. Candidates, in order: the dash's own published move
(`dash_update`'s publish block, where `dash_dir` and `speed` are set - and which is the block the facing fix
touched), `move_axis` (`:9959`, the ordinary physics move), or a state-specific mover such as `move_exact`
(`:11706`) used by climb/dream paths.

Next: instrument the *other* movers on that frame rather than the dash collide path - `move_axis` and the dash
publish - printing `pos` before and after each, and find which one produces 6.83 px instead of 2.83. That is a
narrower search than it has been for several rounds, and it starts from the fact that the collide/correction
path is provably not involved.

### Third narrowing: no corner-correction loop fires on the divergence frame either

Probes inserted before each of the three `for correction in 1..=DASH_CORNER_CORRECTION` loops (labelled A, B, C)
ran on the `g-01` slice. The closest any of them came to the divergence was `(25972,-19246)`; the divergence is
at `(27325,-19637)`, and none of the three printed for it. Together with the earlier result that the frame never
reaches the horizontal `try_dash_collide`, this rules out the whole collide-and-correct path:

- not the collision response (`try_dash_collide` is not reached, and the game's `speed.y` zeroing has no
  counterpart on the simulator's side of that frame),
- not any of the corner corrections (none fires),
- so the extra 4 px of horizontal travel comes from the **plain move** - the transform of speed into position -
  or from a second move in the same frame.

The next measurement is therefore the decisive one and is small: probe `move_axis` (the ordinary physics move at
`sim.rs:9959`) at entry and exit, printing `pos` and `speed` both times, on the divergence frame, and compare the
observed delta with `speed.x * dt`. `speed.x` is `-169.70563`, so the expected horizontal delta is `-2.82843`;
the simulator produced `-6.82843`. Exactly 4 px of unexplained travel is not an accumulation artefact, so either
`move_axis` is called twice with a stale or duplicated speed, or something publishes a `movementCounter` of 4 px
into x before the frame's move.

Also recording the methodological point: the `if horizontal {` grep found **45** sites, and my earlier
`beforeH`-versus-`TC` "contradiction" was an artefact of anchoring two probes in different functions. Probing by
function name beats probing by branch text, and this log has now paid for that lesson twice.

### The extra travel is an exact -4 px move in x, by something other than move_axis, the collide path or the CC loops

Decomposing the dump (which reports `move = dpos + dcounter`) for the divergence frame:

| | x counter at 1965 | at 1966 | dcounter.x | move.x | => dpos.x |
| --- | ---: | ---: | ---: | ---: | ---: |
| game | 0.12650 | 0.29807 | +0.17157 | -2.82843 | **-3** |
| simulator | 0.12650 | 0.29807 | +0.17157 | -6.82843 | **-7** |

So the simulator moved 7 whole pixels left where the game moved 3 - a difference of exactly **4 whole pixels**,
not a fractional accumulation. And `move_axis` (`sim.rs:9958-9961`) is innocent by inspection:

```rust
fn move_axis(p: &mut PlayerSnapshot, map: &mut Map, horizontal: bool) {
    let speed = if horizontal { p.speed.x } else { p.speed.y };
    move_axis_amount(p, map, horizontal, speed * p.frame_delta_time);
}
```

`speed.x * dt = -2.828`, which rounds to the game's -3, and it is called once per axis (`:7158`, `:7161`). So a
**fourth mover** performs an exact 4-pixel horizontal move on that frame - the shape of `MoveHExact(-4)`.

That rules out, in order of the measurements taken: the dash collide response (never reached), the three
`for correction in 1..=DASH_CORNER_CORRECTION` loops (none fires), and now the ordinary physics move. The
candidate list is down to the exact-move helper `move_exact` (`sim.rs:11706`) and whatever calls it, or a
second, state-specific move inside `dash_update`.

Next: list the call sites of `move_exact` and check which of them can run during `StDash` with a 4-pixel
argument; that is now a short list rather than a search.

### move_exact is excluded too; naive_move is the remaining suspect for the exact -4 px

`move_exact` (`sim.rs:11705`) has exactly two call sites - `:10339` and `:10363` - and both are
`move_exact(p, map, false, 1)` / `(..., false, -1)`: vertical, one pixel, so neither can produce a horizontal
move of 4. Excluded.

That leaves `naive_move` (`:9963-9972`), the simulator's `Actor.NaiveMove`, which adds an arbitrary `Vec2` to the
movement remainder and commits the rounded whole pixels:

```rust
fn naive_move(p: &mut PlayerSnapshot, amount: Vec2) {
    p.movement_remainder.x += amount.x;   p.movement_remainder.y += amount.y;
    let move_x = p.movement_remainder.x.round_ties_even();
    let move_y = p.movement_remainder.y.round_ties_even();
    p.movement_remainder.x -= move_x;     p.movement_remainder.y -= move_y;
    p.pos.x += move_x;                    p.pos.y += move_y;
}
```

A 4-pixel x amount through this path would produce exactly the observed `dpos.x = -7` (the -3 from the ordinary
move plus -4 from here). Its call sites are the next thing to list, together with any other move inside
`dash_update` - at this point the search is a handful of callers rather than a code path.

Summary of the elimination for offset 1966, all measured: the collide response is never reached; none of the
three corner-correction loops fires; `move_axis` moves by `speed * dt` and produces the game's -3 on its own;
`move_exact` is vertical-only. The unexplained quantity is one exact 4-pixel horizontal move.

### The last mover is excluded too - so the extra 4 px is a direct position write, and EnforceBounds is the candidate

`naive_move` has exactly one call site, `sim.rs:8277`, inside `dream_dash_update` (`:8275-8284`). The divergence is
in `StDash`, so the dream-dash mover cannot be responsible. With that, every mover in the frame has been
excluded by measurement:

| eliminated | how |
| --- | --- |
| dash collide response (`try_dash_collide`) | never reached on that frame (probe fired 33 times elsewhere) |
| three `for correction in 1..=DASH_CORNER_CORRECTION` loops | none fires (labelled probes A/B/C) |
| `move_axis` | moves by `speed * dt`, which gives the game's -3, and runs once per axis |
| `move_exact` | two call sites, both vertical 1-px |
| `naive_move` | one call site, inside `dream_dash_update` only |

So the unexplained -4 px is not produced by any move; it is a **direct position write**. The survey's table lists
several of those as legitimate *mechanisms* - `OnSquish` probe/restore, `TrySquishWiggle`, `Solid.MoveHExact`
carry, `CassetteBlock.TryActorWiggleUp`, `DreamDashedIntoSolid`, `Respawn`, the intro coroutines - and the one
that can jump a player several pixels without any move is `Level.EnforceBounds`' direct setters
(`sim.rs:11258/11269/11284`, mirroring `player.Left/Right/Top` in `Level.cs:2746`, `:2785`).

A clamp is "legit" as a mechanism and can still fire on the wrong frame: it teleports the player to a bound
whenever the collider is outside it, and during a dash that is exactly the situation where 4 px of horizontal
difference would appear. The same code produced the `roof07` investigation earlier in this log, where the game's
clamp and the simulator's disagreed about when to apply.

Next: probe `enforce_level_bounds` on the divergence frame - print the player rect, `map.bounds`,
`current_room_bounds`, and whether any of the three setters fires - and compare with the game's position on the
same frame. That is one probe and it is the last structural candidate.

### EnforceBounds excluded as well: every structural candidate is now out, so the dash has its own mover

The `enforce_level_bounds` entry probe fired on the divergence frame, and the answer is in its first line:

```
ELB pos=(27328.00,-19637.00) me=(27324.0,-19648.0,8.0,11.0) room=(25016,-21768,2560,3256) state=Dash   <- offset 1965
ELB pos=(27321.00,-19638.00) me=(27317.0,-19649.0,8.0,11.0) room=(25016,-21768,2560,3256) state=Dash   <- offset 1966
```

Two things follow:

1. The clamp runs, but it runs **after** the position is already wrong - `(27321,-19638)` is the simulator's
   post-move position on 1966 while the game's is `(27325,-19637)`. So the 4 px is not a clamp.
2. It could not have clamped anyway: the room's bounds are `y` from `-21768` to `-18512`, and the player is at
   `-19638`, deep inside. The bounds are nowhere near the player on this frame.

So the eliminations now cover the entire frame: collide response, the three corner-correction loops, `move_axis`,
`move_exact`, `naive_move`, and `Level.EnforceBounds`. The only remaining possibility is that the dash's
horizontal travel does **not** go through `move_axis` at all - there is a dash-specific mover (or a position
write inside `dash_update`) that the enumeration did not include.

Next: probe `pos` at the entry and exit of `dash_update` on that frame, and if the -4 appears inside it, walk the
function's own calls. That is the first probe aimed at a *function* rather than at a code path, which is where
this should have started several rounds ago - the movers were enumerated from a grep over `movement_remainder`
and `p.pos`, and `dash_update` reached position through neither.

### FOUND: `close_dash_onto_jump_thru` is the exact-move that produces the unexplained -4 px

`dash_update` (`sim.rs:7757-7876`) has exactly one position-changing call, and its guard is true on the very
frame the divergence appears:

```rust
7763: let dash_coroutine_initial_yield =
7764:     p.dash_dir == Vec2::default() && p.state_timer > DASH_TIME + p.frame_delta_time * 0.5;
...
7786: // Player.cs:4384-4392 closes the player onto a JumpThru it already overlaps
7787: // (any overhang up to six pixels) with an exact move, before the dash jump branches below.
7789: if p.dash_dir.y.abs() < 0.1 {
7790:     close_dash_onto_jump_thru(p, map);
7791: }
...
7803: if (p.state_timer - DASH_TIME).abs() <= p.frame_delta_time * 0.5 {
7804:     p.dash_dir = p.last_aim;              // DashDir is published *here*, after the close
7805-7811:  facing, p.speed = dash_dir * DASH_SPEED
```

On the frame the coroutine publishes `DashDir`, the close runs **before** the publish, so `dash_dir` is still
`Vec2::default()` and `p.dash_dir.y.abs() < 0.1` is `0 < 0.1` - **true**. The helper performs an exact move onto
a JumpThru the player already overlaps, with an overhang of up to six pixels: exactly the shape and magnitude of
the unexplained 4 px, and the only candidate left after every mover, correction, collide path, clamp and
`move_exact`/`naive_move` site was excluded by measurement.

The source has the same step (`Player.cs:4384-4392`), so the defect is not the mechanism but the **condition or
the amount**: which JumpThru counts as "already overlapped", how far the overhang may be, and which direction it
closes in. Note also that this is the same *class* as `DuckCorrect`-style nudges - an exact corrective move whose
precise guard decides whether it fires at all.

Next: read `close_dash_onto_jump_thru` (`sim.rs:7980`) against `Player.cs:4384-4392`, and if the guard differs,
that is the fix; the frame for the check is `7-Summit|0|g-01|134447` offset 1966, where the game moves -3 px and
the simulator -7.

### `close_dash_onto_jump_thru`: the simulator also closes onto `Cloud`, and passes a scalar where the source passes a vector

Side by side:

| source (`Player.cs:4384-4392`) | simulator (`sim.rs:7980-7998`) |
| --- | --- |
| `foreach (JumpThru entity in Tracker.GetEntities<JumpThru>())` - **JumpThru only** | `matches!(entity.kind, EntityKind::JumpThru \| EntityKind::Cloud)` - **Cloud included** |
| `CollideCheck(entity)` | `entity.bounds.intersects(current_player_rect(...))` |
| `base.Bottom - entity.Top <= 6f` | `bottom - entity.bounds.y > 6.0 -> continue` (equivalent) |
| `!DashCorrectCheck(Vector2.UnitY * (entity.Top - base.Bottom))` - argument is a **Vector2** | `!dash_correct_check(p, map, amount as f32)` - argument is a **scalar** |
| `MoveVExact((int)(entity.Top - base.Bottom))` | `move_v_exact(p, map, amount)` with `amount = (bounds.y - bottom) as i32` |

Two concrete divergences, both worth testing on the known frame (`7-Summit|0|g-01|134447` offset 1966, where
the game moves -3 px in x and the simulator -7):

1. **`Cloud` is not a `JumpThru`.** The source's loop only walks `Tracker.GetEntities<JumpThru>()`. Summit's
   `g-01` is full of clouds (the decoded entity list for the room includes `cloud`), so the simulator will close
   the player onto a cloud where the game does not. Note the observed residual is *diagonal* (4 px in x and 1 px
   in y), so this alone may not be the whole story - but it is a genuine, source-backed defect.
2. **The `DashCorrectCheck` argument shape differs.** The source passes the correction offset as a `Vector2`
   (`UnitY * (Top - Bottom)`); the simulator passes a scalar float. If the helper interprets its argument as a
   direction or an offset rather than a magnitude, the check that is supposed to veto the close can invert - and
   that veto is exactly what decides whether this exact move happens at all.

Next, one edit at a time: first restrict the loop to `JumpThru` and measure `--rooms g-01` (expect the `134447`
segment to pass offset 1966 if this is the cause); if it does not move, inspect `dash_correct_check`'s signature
and its use of the scalar against the source's vector form.

### Reverted: `Cloud` IS a `JumpThru` subclass, so the simulator's `| Cloud` was correct all along

Restricting `close_dash_onto_jump_thru`'s loop to `EntityKind::JumpThru` - which looked like a faithful reading of
`Player.cs:4386`'s `Tracker.GetEntities<JumpThru>()` - collapsed the `g-01` segments from 1967/1960 frames to
**117** and the room total from 4,088 to 795. The reason is that Monocle's generic tracker query returns
**subclasses**, and Celeste's `Cloud` derives from `JumpThru`. So the game does close the player onto a cloud, the
simulator's `matches!(kind, JumpThru | Cloud)` was the faithful transcription, and my "divergence 1" was wrong -
a reading error about the semantics of a generic query, not about the code.

Reverted immediately (the build succeeded, so the usual build-failure auto-revert did not fire - reverting a
measured regression is on me, not on the harness). The guard is back to the two-kind form and `crates/` is clean.

Lesson for the log: a source construct that names a base type may implicitly include derived ones
(`GetEntities<T>()`, `CollideCheck<T>()`, `x is T`), and the simulator has to spell the closure out. Before
"fixing" such a list, check whether the missing entry is a subclass - or better, measure it, which is what
caught this in one slice.

That leaves divergence 2 from the previous note unmeasured: the `DashCorrectCheck` argument shape (source passes
`Vector2.UnitY * (Top - Bottom)`, the simulator a scalar). That is the remaining candidate for offset 1966, and it
should be checked by reading `dash_correct_check`'s signature and body before any edit.

### `dash_correct_check`: the veto uses the wrong collider (and cannot move x at all)

| source (`Player.cs:4191-4209`) | simulator (`sim.rs:9670-9676`) |
| --- | --- |
| `DashCorrectCheck(Vector2 add)` - a full vector, both axes | `dash_correct_check(p, map, offset_y: f32)` - y only |
| `base.Collider = hurtbox` - **forced to the standard hurtbox** | `current_player_hurt_rect(p)` - **state-dependent**: ducking `8x4`, `StarFly` `6x6`, else `8x9` |
| `Position += add`, then every `GetComponents<LedgeBlocker>()` | shifts that rect by `offset_y`, then `ledge_blocker_collides` |

Two conclusions:

1. The scalar-versus-vector difference is **benign at this call site**: `close_dash_onto_jump_thru` passes a
   vertical amount, and the helper only ever shifts y, so it cannot produce the 4 px of horizontal travel. That
   retires my "divergence 2" as a cause of the x difference.
2. The **collider** difference is real and consequential in the other direction: the veto is evaluated against a
   different rectangle depending on the player's state, while the source always evaluates it against the plain
   `hurtbox`. A veto that fires in one and not the other decides whether the close happens at all - and the close
   is a `MoveVExact` onto the JumpThru/Cloud, i.e. a **y** move. The measured residual has **1 px of extra y**
   alongside the 4 px of x, which fits: a y shift of one pixel changes what the subsequent horizontal probe hits,
   which cascades into the x difference.

Next (one line): make the veto use the standard hurtbox rather than the state-dependent rect, i.e. mirror
`base.Collider = hurtbox`, and measure the `g-01` slice. Candidate rect to use: `player_hurt_rect(p.pos.x,
p.pos.y)` (`sim.rs:9678`), which is the `8x9` the source's `hurtbox` field holds for a normal player - but check
first whether the source's `hurtbox` is stateful too (it is a field set in the collider table), because that is
exactly the kind of base/derived assumption that was wrong one round ago about `Cloud`.

### Third refutation in the same vein: the source's `hurtbox` IS stateful, so the simulator's veto rect is faithful

Before editing anything, I checked whether `Player.hurtbox` (`Player.cs:481`) is a fixed rect. It is not:

| source assignment | value | simulator `current_player_hurt_rect` |
| --- | --- | --- |
| `:1020` `hurtbox = duckHurtbox` | `8x4 @ (-4,-6)` | ducking -> `8x4 @ (-4,-6)` |
| `:1025`, `:1142`, `:5353` `= normalHurtbox` | `8x9 @ (-4,-11)` | otherwise -> `8x9 @ (-4,-11)` |
| `:5321` `= starFlyHurtbox` | `6x6 @ (-3,-9)` | `StarFly` -> `6x6 @ (-3,-9)` |

So `DashCorrectCheck`'s collider in the simulator is the faithful one, and the "consequential difference" claimed
in the previous note is withdrawn - it would have been a bad change, caught only because the verification step
came before the edit. That is the third base/derived-style misreading in this log (`Cloud` as a `JumpThru`,
`DuckCorrect*`/`MoonLanding` as live constants and functions, and now `hurtbox` as fixed), and the difference
each time was a measurement taken before the edit rather than after.

Where that leaves `7-Summit|0|g-01|134447` offset 1966: every mechanism I have been able to enumerate is
excluded by measurement - the collide response, the three corner-correction loops, `move_axis`, `move_exact`,
`naive_move`, `Level.EnforceBounds`, and now both halves of the dash jump-thru close. Yet the simulator still
moves 4 px further in x and 1 px in y on that frame than the game does. The remaining approach that does not
depend on my enumeration being complete is a **position audit of the frame**: print `pos` at the top of `step`,
after the state callback, after the physics move, and at the end of `step`, so that any position change shows up
as a delta between two labelled phases regardless of which function produced it. That is the systematic version of
what has been done piecemeal for several rounds.

### The four-phase position audit failed on placement - auto-reverted, and the rule is once more "one probe at a time"

The audit patch inserted four labelled probes (top of `step`, after `tick_lift_speed`, after the second
`move_axis`, and at the entry of `enforce_level_bounds`) in a single pass using `IndexOf` arithmetic. The build
failed with `cannot find value p in this scope` four times - i.e. **all four** landed somewhere `p` is not
bound - and the build-failure auto-revert restored the tree and rebuilt it green (356 tests).

That the same class of failure has now happened four times in this log (probe in a different function than
intended; probe inside a guard that cannot be reached; probe inside a `match` scrutinee line; and now four
misplaced probes at once) points at one rule rather than four mistakes: **insert one probe, then build** - and
when the anchor is computed from indices, print the line above and below the insertion point before building, so
the placement is evidence rather than hope. The `tick_lift_speed`/`move_axis` anchors were almost certainly fine
individually; batching them with two index-derived anchors is what made the failure unattributable.

The audit itself remains the right next step for offset 1966, because it does not depend on my enumeration of
movers being complete - which is exactly what has failed to find the extra 4 px so far.

### The audit localised the extra travel: it happens inside `move_axis`, i.e. in `move_axis_amount_inner`

Two literal-anchored probes - after `tick_lift_speed(p);` (call it A2, post-callback) and after
`move_axis(p, map, false);` (A3, post-move) - produced this window around the divergence:

```
A2 postcallback pos=(27329.00,-19564.00) sx=90.00000   sy=224.99995     <- frame ending at 1965
A3 postmove     pos=(27328.00,-19637.00) sx=-169.70563 sy=-169.70563    <- matches the game's 1966 start
A2 postcallback pos=(27328.00,-19637.00) sx=-169.70563 sy=-169.70563    <- frame ending at 1966, before the move
A3 postmove     pos=(27321.00,-19638.00) sx=-169.70563 sy=-169.70563    <- after the move: wrong
```

So between A2 and A3 the position changes by `(-7,-1)` while the speed is `(-169.70563,-169.70563)`, which
should give `(-2.82843,-2.82843)`. The 4 px therefore appear **inside `move_axis`** - and this retires my earlier
"move_axis is innocent" conclusion, which was drawn from reading the wrapper only:

```rust
fn move_axis(p, map, horizontal) {
    let speed = if horizontal { p.speed.x } else { p.speed.y };
    move_axis_amount(p, map, horizontal, speed * p.frame_delta_time);
}
```

The wrapper is indeed innocent; the work happens in `move_axis_amount_inner`, whose collide branch contains the
three correction loops (measured not to fire) **and** direct whole-pixel position writes (the survey's table lists
`10206/10207` and `10230/10248` in this region as `MoveHExact`/`MoveVExact` steps). Those are now the target.

Next: read the collide branch of `move_axis_amount_inner` around those write sites and compare each against its
source counterpart, since one of them is applying a 4-pixel step the game does not.

### CORRECTION: `DuckCorrect*` and `MoonLanding` are LIVE - my name-grep was the wrong instrument

The moveclass workstream checked the vendored file properly and my round-223/224 verdict was wrong. In
`vendor/celeste-fna/Celeste/Player.cs`:

- `DuckCorrectCheck = 4` (`:151`) and `DuckCorrectSlide = 50f` (`:153`) are used at `:3613-3638` with the values
  **inlined**: `:3622 else if (Speed.X == 0f)`, `:3624 for (int num = 4; num > 0; num--)`,
  `:3626 if (CanUnDuckAt(Position + Vector2.UnitX * num))`, `:3628 MoveH(50f * Engine.DeltaTime)`, and the
  mirrored `-num` / `-50f` at `:3631`/`:3633`;
- `MoonLanding` is live: `:6082 public IEnumerator MoonLanding(Vector2 groundPosition)`, called at `:6078`
  (`yield return MoonLanding(start);`), containing `:6090 MoveV(-200f * Engine.DeltaTime)`.

My check was `Select-String -Pattern 'DuckCorrect'` and it found only the declarations, from which I concluded the
constants were dead and even told the workstream twice not to implement them. The instrument was wrong: **the
call sites spell the values inline**, so a name search cannot see the use. Both mechanics exist in the traced
build and are fair game, and `DuckCorrect*` stays the most promising out-of-class lead - a ducked crawl under a
low ceiling cannot creep sideways in the simulator today.

The rule this adds to the log is narrow and useful: **when a constant looks unused, search for its value in the
surrounding function bodies before concluding anything.** This is the fourth base/derived-style misreading of the
session (`Cloud` as a `JumpThru`, `hurtbox` as fixed, `docs/Player.cs` as the source of record, and now values
inlined at call sites) - and the one thing all four have in common is that the evidence was a *pattern match*
rather than a measurement of behaviour.

Status from that workstream: stage 1 of the `FlingBird` work is landed on its branch and verified neutral
(`4f25496`, 42 insertions in `map.rs`, all three traces `0 regressed`, 356 tests). Its recon is detailed - the
bird is the *first* mismatch in exactly three rooms (`j-02`, `j-03`, `j-05`, rows 261936/262656/263755, segments
that replay 18-30 frames today), all 635 `StFlingBird` rows carry `timeRate = 0.8`, the trigger frame's movement
belongs to the old state's `NormalUpdate` with `DoFlingBird`/`FlingBirdBegin` running after the movement pass, and
contact is a `Circle(16)` test rather than the decoded 8x8 rect. Stage 2 is authorised and will be gated on a
`--rooms j-02,j-03,j-05` slice before any full trace.

### The audit arithmetic: something adds roughly -4 to `movement_remainder.x` before the frame's move

The audit put A2 after `tick_lift_speed` (`7113`) and A3 after `move_axis(p, map, false)` (`7161`), with the two
`move_axis` calls at `7158` (x) and `7161` (y) in between. So A2 -> A3 is the **whole frame's** displacement, and
it measured `(-7,-1)` where `speed.x * dt = -2.82843` alone would give -3 - which is exactly what the game moved.

Working backwards from that: for the simulator to move -7 whole pixels, its `movement_remainder.x` entering the
move must have been about -4.126:

```
-4.126 + (-2.82843) = -6.954  ->  round_ties_even = -7      (observed dpos.x = -7)
remainder after      = -6.954 - (-7) = +0.046               (observed dcounter.x ~ +0.17, same order)
```

So something adds roughly **-4 to the x remainder before the frame's move**, and the frame's move then commits
it. The structural list of movers with a 4-scale amount is short: `move_axis_amount` is called with
`to_x - exact_x`-style differences at `9095`, `11229`, `11670`, `11689` and `11700`, all in state-specific paths.

The next probe is therefore a single number: print `movement_remainder` at A2 (`7-Summit|0|g-01|134447`, offset
1966). Whatever it reads answers both "how much" and, together with the call sites, "who" - and it is the last
measurement needed before a fix, because the total displacement is already known to be in `move_axis` and the
remainder is the only input that can inflate it.

Note also for the record: my earlier `beforeH` probe and the `TC` probe cannot both be right (the code between
their anchors is comments only), and since `TC` was anchored on the structural call
`try_dash_collide(p, map, next, true, sign as f32)`, `TC` is the trustworthy one - the horizontal collide branch
is not reached on that frame, which is consistent with the remainder explanation.

### Withdrawn: the remainder was normal at A2 - the extra -4 happens *inside* the x move

The probe printing `movement_remainder` after `tick_lift_speed` (A2) gave, on the divergence frame:

```
REM pos=(27328.00,-19637.00) rem=(0.126499, 0.321927) sx=-169.70563 sy=-169.70563
```

`0.126499` is the ordinary value - it matches the dump's `rustCounter.x = 0.12650` at offset 1965 exactly. So the
previous note's arithmetic ("the x remainder must have been about -4.126 entering the move") is **wrong and
withdrawn**: with `0.126499 + (-2.82843) = -2.70193`, `round_ties_even` gives **-3**, which is precisely the
game's move. `move_axis` should have moved -3.

So the extra 4 px is applied **during** the move and **not** through the remainder. That is a narrow statement
with a narrow consequence: `move_axis_amount_inner`'s per-pixel loop can run at most `|amount| = 3` steps, so the
4 px cannot come from the x `move_axis` call alone. The A2/A3 window contains exactly two moves - `move_axis(p,
map, true)` at `7158` and `move_axis(p, map, false)` at `7161` - so the next probe goes between them, anchored on
the literal line `move_axis(p, map, true);`, where the x move's own result will be visible on its own: either it
moves -7 (and the loop is not bounded the way I read it), or it moves -3 and the remaining -4 comes from
something else in that window.

This is the second time an arithmetic inference from the A2/A3 totals has been retired by one direct
measurement, which is the same lesson as the four base/derived misreadings: measure the intermediate value, do
not infer it from the endpoints.

### SOLVED (localised): the x move is correct; the *vertical* move changes x by 4 - that is the dash corner correction

Probe immediately after `move_axis(p, map, true)` on the divergence frame:

```
A2 (before either move)   pos=(27328.00,-19637.00) rem=(0.126499,0.321927) sx=-169.70563
XM (after the x move)     pos=(27325.00,-19637.00) rem=(0.298066,0.321927) sx=-169.70563
A3 (after the y move)     pos=(27321.00,-19638.00)
dump at offset 1966       game=(27325,-19637); rustCounter.x = 0.29807, rustMove.x = -6.82843
```

Three readings:

1. The x move is **exactly right**: `27328 -> 27325` is the game's -3 px, and the resulting remainder
   `0.298066` matches the dump's `rustCounter.x = 0.29807` to the digit. Nothing is wrong with the horizontal
   move.
2. The remaining discrepancy appears **between XM and A3** - i.e. during `move_axis(p, map, false)`, the
   **vertical** move - which leaves x at 27321 instead of 27325. A vertical move changed x by 4.
3. The only construct that does that is the dash corner correction inside
   `move_axis_amount_inner`'s **vertical** branch: its loop body is `p.pos.y += offset; p.pos.x += sign;`, i.e.
   one correction moves **both** axes - the exact shape and magnitude of the residual.

This also explains why the earlier corner-correction probes appeared not to fire: I read the log with `-Last 8`,
and the divergence frame's line was almost certainly truncated away rather than absent. The lesson already in
this log ("do not infer from endpoints") applies to log windows too - read the whole file or filter by the frame
key, never by the tail.

Next: read the vertical branch's correction loop in `move_axis_amount_inner` (the two loops other than the
horizontal one) and compare its conditions with the source's dash correction, now with the frame known:
`7-Summit|0|g-01|134447` offset 1966, where the game does **not** correct and the simulator does.

### All three correction loops are excluded by their own guards - witness the calls instead

The three `for correction in 1..=DASH_CORNER_CORRECTION` loops and their guards (`sim.rs`):

| loop | guard | this frame |
| --- | --- | --- |
| `10069` (horizontal; body does `p.pos.y += offset; p.pos.x += sign`) | `matches!(state, Dash \| RedDash) && p.speed.y == 0.0 && p.speed.x != 0.0` | during the x move `speed.y = -169.70563`, so the guard is **false** - consistent with XM showing the correct -3 |
| `10172` (downward floor snap; body does `p.pos = (pos.x ∓ correction, pos.y + 1.0)`) | `sign > 0 && p.speed.y > 0.0 && matches!(state, Dash \| RedDash) && !p.dash_started_on_ground` | the dash is upward: `amount = speed.y * dt = -2.828`, so `sign = -1` and the guard is **false** |
| `10183` | mirror of `10172` | same |

So the only construct whose body moves x by up to 4 during a **vertical** move is the floor snap at `10172`/`10183`
- and it cannot fire on this frame because the player is rising. That means the -4 is applied by something that is
not one of these three loops at all, and the cheapest witness for "something" is the call itself.

Next probe: the entry of `move_axis_amount_inner`, printing `horizontal`, `amount`, and `sign`, plus `speed.x` and
`speed.y`. On that frame it will list **every** move the simulator performs, and the one with a non-zero
horizontal flag and a 4-scale amount will name the culprit directly. This is the last probe this investigation
needs: the total displacement is known to happen between XM and A3, all three correction loops are excluded, and
the remainder entering the frame was normal.

### Narrowed to one call: the frame has exactly two moves, so the vertical move's `Rebound` did it

The `move_axis_amount_inner` entry probe lists **every** move the frame performs. On `7-Summit|0|g-01|134447`
offset 1966 there are exactly two, and no move anywhere carries a 4-scale amount:

```
MAI h=true  amount=-2.82843 sign=-1 pos=(27328.00,-19637.00) sx=-169.70563 rem=(0.126499,0.488594)
MAI h=false amount=-2.82843 sign=-1 pos=(27325.00,-19637.00) sx=-169.70563 rem=(0.298066,0.488594)
```

The x move starts at 27328 and leaves 27325 - the game's value, and matching `rustCounter.x`. The **y** move then
starts at 27325 and the frame ends at x = 27321. So the horizontal change happens inside the **vertical** move's
collide branch, and the only construct there that moves the player horizontally is that branch's own
`try_dash_collide(p, map, next, false, sign)` (`sim.rs:10111`) returning `Rebound`, whose arm calls
`rebound(...)`.

That matches `Player.Rebound`, which in the source displaces the player horizontally by a few pixels - the exact
shape and the exact magnitude of the 4 px that has been chased for many rounds. It also fits the guard analysis
that preceded it: the three corner-correction loops are all excluded by their own conditions on this frame, so
the displacement has to be a different construct in the same branch.

Next: read `fn rebound` in the simulator and `Player.Rebound` in the source of record, compare their conditions
and their displacement, and check why the simulator takes the `Rebound` arm here at all - the game evidently does
not rebound on this frame, since its x follows the ordinary move.

### `rebound` excluded too - the x change is inside the vertical move's own per-pixel commit

`fn rebound` (`sim.rs:2535-2552`) never touches the position: it sets `speed = (direction * 120.0, -120.0)`, the
var-jump/auto-jump/wall-slide timers, and calls `enter_normal` when the state is not `Normal`. So if the y move's
`try_dash_collide(p, map, next, false, sign)` had returned `Rebound`, the frame's speed would read `(+/-120,-120)`
at A3 - but A3 measured `sx = -169.70563, sy = -169.70563`, unchanged. `Rebound` is therefore excluded as well.

What the entry probe established, and what remains:

- the frame performs exactly **two** moves (`h=true` then `h=false`), both with `amount = -2.82843`, and **no move
  anywhere carries a 4-scale amount**;
- after the x move the position is `(27325,-19637)` with remainder `0.298066` - the game's values, to the digit;
- the frame ends at x = 27321, so the horizontal change happens **inside the vertical move**;
- every named construct in that branch is now excluded: the three corner-correction loops (guards), `Rebound`
  (does not touch position), `move_exact`/`move_v_exact` (vertical only), `naive_move` (dream dash only), and
  `EnforceBounds` (runs later, player far inside the room).

That leaves the vertical move's own body - specifically its per-pixel commit, where the tail of
`move_axis_amount_inner` writes `p.pos.x = next_x; p.pos.y = next_y;` - or a construct between the collide branch
and that commit. The next probe belongs there: print `pos`, `next_x`/`next_y`, `sign` and `horizontal` immediately
before that assignment, on the same frame. That is now the only unexamined line in the frame's path, and the
investigation has gone from "4 unexplained pixels anywhere in the frame" to "one assignment".

Logged for the next session: the frame key is `7-Summit|0|g-01|134447` offset 1966 (row 136454), the game moves
x by -3 and y by -1, the simulator by -7 and -2, and the discrepancy is in the vertical move.

### The commit-line probe was refused by its own uniqueness guard - here is how to disambiguate it

The literal `        p.pos.x = next_x;` occurs **three** times in `sim.rs`, so the guarded patch exited before
touching anything (the tree is clean). That is the guard working as designed, and it is the same phenomenon as the
45 `if horizontal {` sites: the same line is duplicated across the movers, so a line literal cannot identify a
call site.

Disambiguation recipe for the next attempt, in the order that costs least:

1. find the **enclosing function** (each of the three `p.pos.x = next_x;` sites belongs to a different mover; the
   one wanted is inside `move_axis_amount_inner`), then insert relative to an offset **inside that function's
   span** - `IndexOf('fn move_axis_amount_inner(')`, then `IndexOf('p.pos.x = next_x;', that offset)`, which is a
   single well-defined position;
2. or use a **two-line literal** (the preceding line plus the commit) if the pair is unique;
3. and in all cases print the two lines above and below the insertion point **before** building, which is the rule
   that has caught every placement error since it was adopted.

The frame key for the check remains `7-Summit|0|g-01|134447` offset 1966 (row 136454): the x move lands on the
game's `(27325,-19637)` with remainder `0.298066`, and the vertical move then leaves x at 27321. The probe wanted
prints `horizontal`, `pos`, `next_x`/`next_y`, `sign` and the speeds immediately before the commit, which is the
last unexamined line in the frame's path.

### The vertical move never commits at the divergence frame - it returns early, and the x change is in that path

The commit-line probe (inserted inside `move_axis_amount_inner`'s span, before `p.pos.x = next_x;`) ran over the
`g-01` slice. Every `h=false` commit in the log sits at earlier frames (`y` between -19584 and -19570); there is
**no** `h=false` commit at the divergence frame, whose entry was witnessed as
`MAI h=false amount=-2.82843 sign=-1 pos=(27325.00,-19637.00) sx=-169.70563`.

So on that frame the vertical move **returns before the per-pixel commit**, yet the frame ends at x = 27321. The
4 px is therefore written inside one of the early-return paths of the vertical branch. That is a short list, and
everything already excluded narrows it further:

- the two floor-snap correction loops (`10172`, `10183`): excluded by their own guards (`sign > 0`, and
  `p.speed.y > 0.0` - the dash is upward);
- `Rebound`: excluded, it does not touch the position and would have changed the speeds;
- `rebound()`'s siblings in the same `match` (`Ignore` returns without writing) and the `HitSquash` write (state
  only);
- `move_v_exact`/`move_h_exact` and `naive_move`: not reachable from this path.

What remains in that branch's early returns is a position write that is not one of the named helpers - which is
consistent with the last several rounds' pattern of the culprit being a construct whose name never appeared in my
greps. The next probe belongs at the **top of the vertical branch's collide handling** and at each `return`, or
simply: print `pos` at the entry and at every exit of the branch, and the exit whose delta is 4 names it.

Frame key unchanged: `7-Summit|0|g-01|134447` offset 1966 (row 136454); game x -3 / y -1, simulator x -7 / y -2.
