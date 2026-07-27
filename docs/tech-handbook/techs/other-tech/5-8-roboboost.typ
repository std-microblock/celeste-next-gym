#import "../../template.typ": tech, evidence

#tech(
  id: "5.8",
  title-zh: "Roboboost",
  title-en: "Roboboost",
  status: "unimplemented",
  description-zh: [Roboboost 在移动 Solid 上完成六帧 Hyper Bunnyhop，并在平台移动与 LiftSpeed 保留窗口内衔接反向 Cornerboost。当前模拟器已有这些基础机制，但缺少可由真实 Everest 加载的同源移动方块实体及严格子像素场景，因此暂不宣称实现。],
  description-en: [A roboboost performs a six-frame hyper bunnyhop on a moving Solid and chains a reverse cornerboost within the platform movement and retained LiftSpeed window. The simulator has the component mechanics, but no source-matched moving-block entity loadable by real Everest or strict subpixel scenario, so it remains unimplemented.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste/Actor.cs; related moving-block entity source],
    symbol: [Player.SuperJump; Player.ClimbJump; Player.OnCollideH; Actor.LiftSpeed; Actor.Update; Solid.MoveHExact; Solid.MoveVExact],
    snippet: raw(block: true, lang: "cs", "// Hyper conversion\nif (Ducking) {\n    Ducking = false;\n    Speed.X *= 1.25f;\n    Speed.Y *= .5f;\n}\n// Reverse climb jump and corner retention\nSpeed.X += JumpHBoost * moveX;\nwallSpeedRetained = Speed.X;\nwallSpeedRetentionTimer = WallSpeedRetentionTime;\nSpeed.X = 0;"),
    note: [基础顺序是蹲姿 SuperJump 先产生 Hyper，六帧内落地保速后再次起跳；移动 Solid 在玩家更新前/后 carry、push 并写入 LiftSpeed，随后反向 ClimbJump 的 +40 与墙碰撞 retention 共同决定输出。完整技巧还依赖特定实体速度、碰撞边缘与子像素余数。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [PlayerSnapshot.last_lift_speed; advance_moving_solids; super_jump; climb_jump; update_wall_speed_retention], note: [Rust 已覆盖 0.16 秒 LiftSpeed、Hyper、ClimbJump 与 Cornerboost retention，但当前 MovingSolid 是模拟器夹具实体，物理仓库没有对应真实 Everest mod entity 实现，不能据此伪造真实对照。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [moving_solid_jump_combines_carrying_with_same_frame_lift_boost; cornerboost_climb_jump_stores_jump_boost_before_clearing_wall_top], note: [现有回归只证明组成机制，不构成 Roboboost 的端到端 verdict。]),
  e2e-evidence: none,
  candidate-e2e: none,
)
