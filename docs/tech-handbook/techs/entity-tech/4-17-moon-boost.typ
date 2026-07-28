#import "../../template.typ": tech, evidence

#tech(
  id: "4.17",
  title-zh: "Moon Boost",
  title-en: "Moon Boost",
  status: "implemented",
  description-zh: [以 Super 或 Demo Hyper 撞月亮方块顶部 3 像素，墙角修正会把玩家推到方块顶面，同时方块运动产生大量 liftboost。],
  description-en: [Hitting the top three pixels of a moon block with a super or demohyper combines corner correction onto the block with strong movement liftboost.],
  source-evidence: evidence(
    path: [Celeste/MoveBlock.cs; Source/Player/Player.cs],
    symbol: [MoveBlock.Controller; Player.Jump; Player.LiftBoost],
    snippet: raw(block: true, lang: "cs", "targetAngle = homeAngle + MathF.PI / 4f * angleSteerSign * Input.MoveY.Value;\nspeed = Calc.Approach(speed, targetSpeed, 300f * Engine.DeltaTime);\nVector2 vector = Calc.AngleToVector(angle, speed);\nLiftSpeed = vector;\n...\nSpeed += LiftBoost;"),
    note: [横向 MoveBlock 在 rider 连续站立 0.2 秒后按 MoveY 转向最多 45°，以每秒 300 加速到 60，并在同一实体更新写入二维 LiftSpeed；玩家跳跃随后叠加该 lift，因此向上转向帧可把普通 -105 跳速进一步抬高。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [advance_move_blocks / move_move_block_axis], note: [实现 rider 激活、0.2 秒等待、45° steering、300/s 加速、整数 movementCounter carry 与二维 LiftSpeed。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/sim.rs], symbol: [vanilla_move_block_round_trips_through_celeste_binary / moon_block_steering_writes_diagonal_lift_for_a_jump / move_block_runtime_keeps_split_simulation_composable]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.17-moon-boost.ts], symbol: [entity-4.17-moon-boost], note: [独立可转向 MoveBlock 场景在真实 Everest 中观察到右上位移和带负 Y lift 的跳跃。91 个状态的 position、speed、state、facing、dashes、stamina、grounded、ducking、death 全部逐帧一致，最大 position/speed 误差均为 0；完整 trace 已生成 MP4、poster 与带进程/nonce 的 manifest。]),
  candidate-e2e: none,
)
