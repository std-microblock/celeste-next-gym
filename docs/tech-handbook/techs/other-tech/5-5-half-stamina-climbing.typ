#import "../../template.typ": tech, evidence

#tech(
  id: "5.5",
  title-zh: "半体力攀爬",
  title-en: "Half Stamina Climbing",
  status: "implemented",
  description-zh: [Neutral 攀跳后先在 wallboost 窗口输入离墙方向返还体力，再利用仍处于攀跳探测距离内的一帧反向抓墙攀跳；两次攀跳净消耗一次 27.5 体力。],
  description-en: [After a neutral climb jump, press away to refund its wallboost cost, then reverse into a close-wall climb jump while still in probe range; two climb jumps net one 27.5 stamina cost.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.Update; Player.NormalUpdate; Player.ClimbJump],
    snippet: raw(block: true, lang: "cs", "if (wallBoostTimer > 0) {\n    wallBoostTimer -= Engine.DeltaTime;\n    if (moveX == wallBoostDir) {\n        Speed.X = WallJumpHSpeed * moveX;\n        Stamina += ClimbJumpCost;\n        wallBoostTimer = 0;\n    }\n}\n...\nif (!onGround) Stamina -= ClimbJumpCost;\nJump(false, false);\nif (moveX == 0) {\n    wallBoostDir = -(int) Facing;\n    wallBoostTimer = ClimbJumpBoostTime;\n}"),
    note: [ClimbJump 先扣 27.5 并打开 0.2 秒 wallboost；下一帧 Player.Update 在 NormalUpdate 的近墙攀跳判定之前返还 27.5、设置 130 离墙速度，因此同帧第二次攀跳再扣 27.5，净值仍只有一次攀跳成本。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [step; update_wall_boost; normal_update; climb_jump],
    note: [Rust 保持 Player.Update 先结算 wallboost、再执行 NormalUpdate 攀跳的顺序；缓存 move_x 与两像素 ClimbCheck 让返还和第二次攀跳在同一状态帧发生。],
  ),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [half_stamina_climbing_chains_wallboost_into_close_wall_climb_jump], note: [三输入帧从 80 体力开始，第一攀跳降到 52.5，第二攀跳后仍为 52.5，并断言 Normal、朝向、冲刺数、接地、蹲伏与死亡字段。]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.5-half-stamina-climbing.ts], symbol: [other-5.5-half-stamina-climbing; verifyHalfStaminaClimbing], note: [专属 MapPart 的直墙场景记录 4 个真实状态；wallboost 返还与第二次近墙攀跳后体力仍为 52.5，九类核心字段逐帧一致，最大位置误差 0、速度误差 0.000008。]),
  candidate-e2e: none,
)
