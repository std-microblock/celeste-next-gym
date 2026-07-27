#import "../../template.typ": tech, evidence

#tech(
  id: "1.10",
  title-zh: "平台动量加成",
  title-en: "Liftboost",
  status: "implemented",
  description-zh: [移动实体把位移速度写入 LiftSpeed，并保留 0.16 秒；LiftBoost 将横向限制到 ±250、向上限制到 -130、向下清零。玩家跳跃、墙跳、攀爬跳、Super 与冲刺会按各自动作规则消费该速度。],
  description-en: [Moving entities write their displacement velocity into LiftSpeed and retain it for 0.16 seconds. LiftBoost clamps horizontal speed to ±250, upward speed to -130, and removes downward speed; jumps, wall jumps, climb jumps, supers, and dashes consume it according to their action rules.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste.ZipMover.Sequence; Celeste.Platform.MoveV],
    symbol: [Player.LiftBoost; Player.Jump; Player.WallJump; Player.NormalUpdate; Platform.LiftSpeed],
    snippet: raw(block: true, lang: "cs", "Vector2 val = LiftSpeed;\nif (Math.Abs(val.X) > LiftXCap)\n    val.X = LiftXCap * Math.Sign(val.X);\nif (val.Y > 0) val.Y = 0;\nelse if (val.Y < LiftYCap) val.Y = LiftYCap;\n...\nSpeed.Y = JumpSpeed;\nSpeed += LiftBoost;\n...\nLiftSpeed.Y = moveV / Engine.DeltaTime;"),
    note: [真实实体顺序为 Player 先更新、ZipMover 后移动；因此平台本帧写入的 LiftSpeed 在下一帧玩家动作前可见。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [EntityKind.ZipMover; ZipMoverSnapshot; advance_zip_movers; lift_boost; tick_lift_speed]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [zip_mover_uses_source_wait_yield_and_sine_outbound_phases; zip_mover_previous_frame_carry_writes_lift_speed_for_next_player_update; zip_mover_runs_after_player_and_matches_real_vertical_push_order; jump_adds_retained_lift_boost_before_caching_variable_jump_speed]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [mechanics-liftboost-zip-jump; verifyZipMoverLiftboost], note: [真实 25 帧 ZipMover 场景确认平台上一帧写入 -29.575138 的纵向 LiftSpeed，下一帧跳跃速度为 -134.57513；九类核心字段最大位置与速度误差均为 0。]),
  candidate-e2e: none,
)
