#import "../../template.typ": tech, evidence

#tech(
  id: "3.12.1",
  title-zh: "Cornerboost Wallboost",
  title-en: "Cornerboost Wallboost (cobwob)",
  status: "implemented",
  description-zh: [先用 neutral 攀跳完成 Cornerboost，再在 wallboost 窗口内按反方向，把动作转为 wallboost；原有高速会被墙跳速度覆盖。],
  description-en: [A neutral cornerboost followed by an away input becomes a wallboost, replacing the incoming speed with wallkick speed.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.ClimbJump; Player.Update],
    snippet: raw(block: true, lang: "cs", "if (moveX == 0) {\n    wallBoostDir = -(int) Facing;\n    wallBoostTimer = ClimbJumpBoostTime;\n}\n...\nif (wallBoostTimer > 0 && moveX == wallBoostDir) {\n    Speed.X = WallJumpHSpeed * moveX;\n    Stamina += ClimbJumpCost;\n    wallBoostTimer = 0;\n}"),
    note: [Neutral ClimbJump 打开 0.2 秒反向输入窗口；后续 Update 用 +/-130 覆盖 Cornerboost 保留速度并返还 27.5 体力，所以转换后的速度不是两者叠加。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [climb_jump; update_wall_boost]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [cornerboost_wallboost_overwrites_retained_speed_with_wallkick_speed]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/cornerboost-wallboost.ts], symbol: [cornerboost-wallboost; verifyCobwob], note: [独立墙角 MapPart 共 13 个真实状态；state 1 保留速度大于 140、wallBoostTimer 大于 0.19、体力 82.5，state 3 体力回到 110 且水平速度约 -123。最大位置误差 0、速度误差 0.000001，其余字段逐帧一致。]),
  candidate-e2e: none,
)
