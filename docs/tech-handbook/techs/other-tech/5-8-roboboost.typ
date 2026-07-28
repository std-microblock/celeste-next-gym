#import "../../template.typ": tech, evidence

#tech(
  id: "5.8",
  title-zh: "Roboboost",
  title-en: "Roboboost",
  status: "unimplemented",
  description-zh: [Roboboost 在移动 Solid 上完成六帧 Hyper Bunnyhop，并在平台移动与 LiftSpeed 保留窗口内衔接反向 Cornerboost。Rust 已覆盖组成机制，现新增真实 vanilla MoveBlock 独立候选；完整组合未通过真实 Everest 前保持未实现。],
  description-en: [A roboboost performs a six-frame hyper bunnyhop on a moving Solid and chains a reverse cornerboost inside the platform and retained-LiftSpeed window. Rust covers the component mechanics and now has an independent vanilla MoveBlock candidate; the full composition remains unimplemented until real Everest validates it.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste/Actor.cs; Celeste/Solid.cs; Celeste/MoveBlock.cs],
    symbol: [Player.SuperJump; Player.ClimbJump; Player.OnCollideH; Actor.LiftSpeed; Solid.MoveHExact; MoveBlock.Controller],
    snippet: raw(block: true, lang: "cs", "if (Ducking) {\n    Ducking = false;\n    Speed.X *= 1.25f;\n    Speed.Y *= .5f;\n}\n...\nSpeed.X += JumpHBoost * moveX;\nwallSpeedRetained = Speed.X;\nwallSpeedRetentionTimer = WallSpeedRetentionTime;"),
    note: [SuperJump 先完成 Hyper 转换；Solid 的 carry/push 在 Player 更新前写入 LiftSpeed，Actor 保留最后非零值 0.16 秒。反向 ClimbJump 的 +40 与随后的 OnCollideH retained speed 决定 Cornerboost 输出。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [PlayerSnapshot.last_lift_speed; advance_move_blocks; advance_moving_solids; super_jump; climb_jump; update_wall_speed_retention], note: [Rust 保留 MoveBlock runtime、carry/push、0.16 秒 LiftSpeed、Hyper、ClimbJump 与墙速返还；尚缺同一真实轨迹对整段组合的确认。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [moving_solid_jump_combines_carrying_with_same_frame_lift_boost; cornerboost_climb_jump_stores_jump_boost_before_clearing_wall_top; move_block_runtime_keeps_split_simulation_composable; roboboost_fixture_restores_climb_jump_speed_before_reversing_input], note: [精确回归分别锁定同帧 carry+lift jump、反向 ClimbJump 后 130 retained speed、MoveBlock 分段重放，以及竖直 MoveBlock 场景中第 52 帧的墙速保留与第 55 帧恢复的大于 300 速度；这些仍是组成证据。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.8-roboboost.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs; crates/celeste-physics/examples/compare_real_trace.rs], symbol: [other-5.8-roboboost; SnapshotCapture.Capture; compare_real_trace], note: [独立 MapPart 使用真实向上 MoveBlock 与 8px 网格角墙。方块先在 0.2 秒等待后提供上升 LiftSpeed；第 51 个输入帧的 ClimbJump 越过墙顶，仍在墙速保留窗口内恢复大于 300，随后反向输入。该竖直 MoveBlock 场景与源代码的转向判定对齐；必须在新的隔离 E2E 运行中保留由运行 manifest 的 `traces` 条目绑定的轨迹，历史共享 `.tmp` 轨迹不构成证据。]),
)
