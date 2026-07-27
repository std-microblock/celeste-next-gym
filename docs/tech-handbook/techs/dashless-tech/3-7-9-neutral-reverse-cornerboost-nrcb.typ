#import "../../template.typ": tech, evidence

#tech(
  id: "3.7.9",
  title-zh: "中性反向 Cornerboost",
  title-en: "Neutral Reverse Cornerboost (nrcb)",
  status: "implemented",
  description-zh: [面向墙角并无方向攀跳可避免反向 Cornerboost 的减速，再利用 11 帧 wallboost 窗口重设水平速度。],
  description-en: [A neutral reverse cornerboost avoids the backward speed loss and can convert into a wallboost within its eleven-frame window.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.ClimbJump; Player.Update],
    snippet: raw(block: true, lang: "cs", "if (moveX == 0) {\n    wallBoostDir = -(int)Facing;\n    wallBoostTimer = ClimbJumpBoostTime;\n}\n...\nif (wallBoostTimer > 0 && moveX == wallBoostDir) {\n    Speed.X = WallJumpHSpeed * moveX;\n    Stamina += ClimbJumpCost;\n    wallBoostTimer = 0;\n}"),
    note: [无方向攀跳不会施加反向 40 加速，而是打开 0.2 秒 wallboost 窗口；窗口内转向离墙时将水平速度设为 130 并退回 27.5 体力。该顺序解释了先保速、后转换的 NRCB。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [climb_jump; update_wall_boost]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [neutral_reverse_cornerboost_keeps_speed_then_converts_within_wallboost_window]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/neutral-reverse-cornerboost.ts], symbol: [neutral-reverse-cornerboost; verifyNeutralReverseCornerboost], note: [独立墙角 MapPart 记录 21 个真实状态；state 1 速度 149.16664/-105、体力 82.5 且 wallBoostTimer 为 0.2，state 3 转换为 125.66666/-105 并恢复 110 体力。九类核心字段逐帧一致，最大位置与速度误差均为 0。]),
  candidate-e2e: none,
)
