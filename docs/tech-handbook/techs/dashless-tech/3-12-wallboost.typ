#import "../../template.typ": tech, evidence

#tech(
  id: "3.12",
  title-zh: "Wallboost 墙面加速",
  title-en: "Wallboost",
  status: "implemented",
  description-zh: [Neutral 攀跳后的 11 帧内按离墙方向，会把动作追溯转换为墙跳、返还体力并设置墙跳水平速度。],
  description-en: [Pressing away within eleven frames of a neutral climb jump retroactively converts it to a wallkick, refunding stamina and setting wallkick speed.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.Update; Player.ClimbJump],
    snippet: raw(block: true, lang: "cs", "if (wallBoostTimer > 0) {\n    wallBoostTimer -= Engine.DeltaTime;\n    if (moveX == wallBoostDir) {\n        Speed.X = WallJumpHSpeed * moveX;\n        Stamina += ClimbJumpCost;\n        wallBoostTimer = 0;\n    }\n}\n...\nif (moveX == 0) {\n    wallBoostDir = -(int) Facing;\n    wallBoostTimer = ClimbJumpBoostTime;\n}"),
    note: [Neutral climb jump 打开 0.2 秒窗口并保存离墙方向。Player.Update 使用缓存的 moveX 命中窗口时设置 130 水平速度、返还 27.5 体力并关闭计时器。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [PlayerSnapshot.move_x; climb_update; update_wall_boost]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [neutral_climb_jump_converts_to_wallboost_and_refunds_stamina]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [wallboost], note: [真实 Normal→Climb→neutral climb jump→wallboost 共 13 个状态帧；九类核心字段逐帧一致，max position error 0，max speed error 0.000046。]),
  candidate-e2e: none,
)
