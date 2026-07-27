#import "../../template.typ": tech, evidence

#tech(
  id: "2.4",
  title-zh: "Wavedash 波冲",
  title-en: "Wavedash",
  status: "implemented",
  description-zh: [从空中向下斜冲刺落地并及时起跳，得到与 Hyper 相同的速度和高度，但对起步地面长度要求更低。],
  description-en: [Starting a down-diagonal dash in the air, landing, and jumping in time reproduces hyper speed and height with less runway.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.OnCollideV; Player.DashUpdate; Player.SuperJump],
    snippet: raw(block: true, lang: "cs", "if (DashDir.X != 0 && DashDir.Y > 0 && Speed.Y > 0) {\n    DashDir.Y = 0;\n    Speed.Y = 0;\n    Speed.X *= DodgeSlideSpeedMult;\n    Ducking = true;\n}\n...\nif (DashDir.Y == 0 && Input.Jump.Pressed && jumpGraceTimer > 0)\n    SuperJump();"),
    note: [空中下斜冲刺落地时转成 1.2 倍水平蹲伏滑行，再由 DashUpdate 跳成 Hyper。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [move_exact; dash_update; super_jump]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [wavedash_landing_converts_down_diagonal_dash_to_hyper]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [wavedash], note: [真实落地帧 203.646 水平速度，下一帧 325/-52.5 且 Dash 已恢复；最大误差 0。]),
  candidate-e2e: none,
)
