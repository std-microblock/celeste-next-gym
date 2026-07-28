#import "../../template.typ": tech, evidence

#tech(
  id: "2.10",
  title-zh: "Wallbounce 墙反",
  title-en: "Wallbounce (wounce, wb)",
  status: "implemented",
  description-zh: [向上冲刺贴近墙面时，在 Dash Attack 窗口内按跳可向外弹出，获得约 170 水平速度和 160 向上速度。],
  description-en: [Jumping beside a wall during an upward dash attack launches a wallbounce at about 170 horizontal and 160 upward speed.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.NormalUpdate; Player.DashUpdate; Player.SuperWallJump],
    snippet: raw(block: true, lang: "cs", "if (canUnduck && WallJumpCheck(-1)) {\n    ...\n    else if (DashAttacking && DashDir.X == 0 && DashDir.Y == -1)\n        SuperWallJump(1);\n    else\n        WallJump(1);\n}\n...\nSpeed.X = SuperWallJumpH * dir;\nSpeed.Y = SuperWallJumpSpeed;"),
    note: [DashBegin 打开的 DashAttacking 窗口为 0.3 秒，而普通 Dash 状态只持续 0.15 秒；因此上冲协程已经回到 Normal 后，NormalUpdate 仍会优先把近墙跳跃分派给 SuperWallJump，得到 170 水平、-160 垂直，而不是普通墙跳的 130/-105。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [normal_update; dash_update; super_wall_jump]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; scripts/e2e-real/timelines.ts; scripts/e2e-real/test/timeline-regressions.test.ts; tests/timelines/delayed-wallbounce.json], symbol: [wallbounce_sets_super_wall_jump_speed_and_var_window; discoverTimelineFixtures; runTimelineRegression]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/area-1/wallbounce.ts; scripts/e2e-real/scenarios/playground/delayed-wallbounce.ts], symbol: [wallbounce; delayed-wallbounce], note: [常规墙反的真实关键帧为 -170/-160，max speed error 0.000038。新增延迟场景在 Dash 状态结束三帧后、dashAttackTimer 仍约 0.08333 时按跳；25 个真实状态的 position 最大误差 0、speed 最大误差 0.000023，其余七类核心字段逐帧一致。]),
  candidate-e2e: none,
)
