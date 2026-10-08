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
| `max_dashes` | derived | `Player.MaxDashes` -> `PlayerInventory.Dashes` for the segment's `Session.Area.ID` (`AreaData`), tightened by the largest `Dashes` the trace reports in the segment. |
| `no_refills` | derived | `PlayerInventory.NoRefills` for the segment's `Session.Area.ID` (`AreaData`). |
| `frame_delta_time` | derived | `#[serde(skip)]` on the wire type. `Simulator::step` recomputes it every frame as the supplied `rawDt` bits times `time_rate`. |
| `state_timer` | derived | in the Dash state, `PlayerSnapshot::restore_dash_phase` rebuilds the simulator's dash clock from `p.dashAttackTimer`: Celeste times the dash with `DashCoroutine` (`Player.cs:4465-4567`), not a `StateMachine.Timer`, and `dashAttackTimer` (`Player.cs:4296`, decremented per unfrozen frame at `Player.cs:1577-1580`) counts exactly those frames. |
| `wind` | derived | top-level `wind` (`Celeste.Level.Wind`, `Level.cs:149`). Restored verbatim from the anchor row so the room segment continues the source's own ramp: `WindController.Update` rewrites it with `Calc.Approach(level.Wind, targetSpeed, 1000f * Engine.DeltaTime)` (`WindController.cs:194`) and displaces the Player's `WindMover` component (`Player.cs:1180`) by `level.Wind * 0.1f * Engine.DeltaTime` (`WindController.cs:199-201`). |
| `wind_target` | derived | top-level `windTarget` (`WindController.targetSpeed`, `WindController.cs:47`, read by reflection through the private `Level.windController` field at `Level.cs:101`). The replayed ramp needs the source's own target, which a room segment cannot reconstruct. |
| `ducking` | derived | top-level `ducking` (`Player.Ducking`, `Player.cs:1005-1028`). A computed property over `Monocle.Entity.Collider` (`Monocle/Entity.cs:73`), so it is not a declared field; the exporter also writes the active collider as `collider` = `[absoluteLeft, absoluteTop, width, height]` (`Monocle/Collider.cs:229,205,12,14`). |
| `freeze_timer` | derived | top-level `freezeTimer` (`Monocle.Engine.FreezeTimer`, `Monocle/Engine.cs:28`). While positive `Engine.Update` only decrements it and skips `Scene.Update` entirely (`Engine.cs:266-269`); the exporter writes the post-decrement value, which is exactly the snapshot state `Simulator::step` reads at the top of the next frame. |
| `can_dream_dash` | derived | top-level `inventory.DreamDash` (`Celeste.Session.Inventory`, `Session.cs:35`, `PlayerInventory.cs:24`). `Player.Inventory` forwards it (`Player.cs:956-966`) and the source reads it at `Player.cs:3420` and `4500`; restoring it removes the need to infer the flag from `dreamDashCanEndTimer`. |
| `core_mode` | derived | top-level `coreMode` (`Celeste.Session.CoreMode`, `Session.cs:111`; `None = 0, Hot = 1, Cold = 2` per `Session.cs:22-27`). Session state, so the base-chain dump cannot see it; the Core's ice factor (`Player.cs:3681-3684`, `if (onGround && level.CoreMode == Cold) num2 *= 0.3f`) and the `CoreModeListener` entities read it. Without it every Core room replays as `CoreMode::None`. |
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
| `intro_phase` | none | `Player.Intro*` coroutine progress. The intro state callbacks (`Player.cs` IntroWalk/IntroJump/IntroWakeUp/IntroThinkForABit) live in the state machine's enumerators, and the sprite clock lives on the `Monocle.Sprite` component, so no `Player` field carries it. The simulator reconstructs the phase once from the anchor's state, position and facing (`intro_resume`) instead. |
| `intro_phase_ready` | none | `Player.Intro*` coroutine progress. The intro state callbacks (`Player.cs` IntroWalk/IntroJump/IntroWakeUp/IntroThinkForABit) live in the state machine's enumerators, and the sprite clock lives on the `Monocle.Sprite` component, so no `Player` field carries it. The simulator reconstructs the phase once from the anchor's state, position and facing (`intro_resume`) instead. |
| `intro_sprite_frame` | none | `Player.Intro*` coroutine progress. The intro state callbacks (`Player.cs` IntroWalk/IntroJump/IntroWakeUp/IntroThinkForABit) live in the state machine's enumerators, and the sprite clock lives on the `Monocle.Sprite` component, so no `Player` field carries it. The simulator reconstructs the phase once from the anchor's state, position and facing (`intro_resume`) instead. |
| `intro_sprite_timer` | none | `Player.Intro*` coroutine progress. The intro state callbacks (`Player.cs` IntroWalk/IntroJump/IntroWakeUp/IntroThinkForABit) live in the state machine's enumerators, and the sprite clock lives on the `Monocle.Sprite` component, so no `Player` field carries it. The simulator reconstructs the phase once from the anchor's state, position and facing (`intro_resume`) instead. |
| `intro_start` | none | `Player.Intro*` coroutine progress. The intro state callbacks (`Player.cs` IntroWalk/IntroJump/IntroWakeUp/IntroThinkForABit) live in the state machine's enumerators, and the sprite clock lives on the `Monocle.Sprite` component, so no `Player` field carries it. The simulator reconstructs the phase once from the anchor's state, position and facing (`intro_resume`) instead. |
| `intro_timer` | none | `Player.Intro*` coroutine progress. The intro state callbacks (`Player.cs` IntroWalk/IntroJump/IntroWakeUp/IntroThinkForABit) live in the state machine's enumerators, and the sprite clock lives on the `Monocle.Sprite` component, so no `Player` field carries it. The simulator reconstructs the phase once from the anchor's state, position and facing (`intro_resume`) instead. |
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

Declared `PlayerSnapshot` fields: 160. Restored from a `p` key: 61. Derived: 14. Unrestored: 86. Missing from table: []. Stale table entries: [].
