#import "../../template.typ": tech, evidence

#tech(
  id: "5.8",
  title-zh: "Roboboost",
  title-en: "Roboboost",
  status: "implemented",
  description-zh: [Roboboost 在移动 Solid 上完成六帧 Hyper Bunnyhop，并在平台移动与 LiftSpeed 保留窗口内衔接反向 Cornerboost。Rust 现与原版竖直 MoveBlock 的转向判定对齐，且独立真实 Everest 轨迹已逐帧验证完整组合。],
  description-en: [A roboboost performs a six-frame hyper bunnyhop on a moving Solid and chains a reverse cornerboost inside the platform and retained-LiftSpeed window. Rust now matches vanilla vertical MoveBlock steering, and an independent real Everest trace validates the full sequence frame by frame.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste/Actor.cs; Celeste/Solid.cs; Celeste/MoveBlock.cs],
    symbol: [Player.SuperJump; Player.ClimbJump; Player.OnCollideH; Actor.LiftSpeed; Solid.MoveHExact; MoveBlock.Controller],
    snippet: raw(block: true, lang: "cs", "if (Ducking) {\n    Ducking = false;\n    Speed.X *= 1.25f;\n    Speed.Y *= .5f;\n}\n...\nSpeed.X += JumpHBoost * moveX;\nwallSpeedRetained = Speed.X;\nwallSpeedRetentionTimer = WallSpeedRetentionTime;\n...\nbool flag = ((direction != Directions.Right && direction != Directions.Left)\n  ? HasPlayerClimbing()\n  : HasPlayerOnTop());"),
    note: [SuperJump 先完成 Hyper 转换；Solid 的 carry/push 在 Player 更新前写入 LiftSpeed，Actor 保留最后非零值 0.16 秒。反向 ClimbJump 的 +40 与随后的 OnCollideH retained speed 决定 Cornerboost 输出。MoveBlock.Controller 对竖直方向只接受侧面攀爬的转向输入；顶乘仅适用于水平 block，因此不会过早消耗竖直 block 的 noSteerTimer。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [PlayerSnapshot.last_lift_speed; advance_move_blocks; advance_moving_solids; super_jump; climb_jump; update_wall_speed_retention], note: [Rust 保留 MoveBlock runtime、carry/push、0.16 秒 LiftSpeed、Hyper、ClimbJump 与墙速返还；竖直 block 的 steering player 明确为攀爬者，完整组合已由同一真实轨迹确认。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [moving_solid_jump_combines_carrying_with_same_frame_lift_boost; cornerboost_climb_jump_stores_jump_boost_before_clearing_wall_top; move_block_runtime_keeps_split_simulation_composable; roboboost_fixture_restores_climb_jump_speed_before_reversing_input], note: [精确回归分别锁定同帧 carry+lift jump、反向 ClimbJump 后 130 retained speed、MoveBlock 分段重放，以及竖直 MoveBlock 场景中第 52 帧的墙速保留与第 55 帧恢复的大于 300 速度；这些仍是组成证据。]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.8-roboboost.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs; crates/celeste-physics/examples/compare_real_trace.rs; .tmp/e2e-runs/2026-07-28T20-05-26.641Z-107664-29be3ea9-1465-41e9-a363-b7855b90d74e/manifest.json], symbol: [other-5.8-roboboost; SnapshotCapture.Capture; compare_real_trace], note: [2026-07-28 的隔离真实 Everest run `2026-07-28T20-05-26.641Z-107664-29be3ea9-1465-41e9-a363-b7855b90d74e` 使用 manifest 的 `traces/other-5.8-roboboost.json` 绑定 91 帧轨迹；所有九个核心字段逐帧验证，最大 position 误差为 0、speed 误差为 0.000041（均不超过 0.01）。manifest 以 cleanup-finished 完成，nonce `cc6d57d6-f325-4338-8bcc-8ab40344f790` 与本次 spawned Celeste PID `106288` 认证匹配。],),
  candidate-e2e: none,
)
