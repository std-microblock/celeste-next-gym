#import "../../template.typ": tech, evidence

#tech(
  id: "2.7",
  title-zh: "Superwave",
  title-en: "Superwave",
  status: "unimplemented",
  description-zh: [先完成延长 Super，随后立刻接反向 Wavedash，以组合方式保留冲刺并改变高速移动方向。],
  description-en: [A superwave chains an extended super directly into a reverse wavedash, commonly using clouds or crumble blocks as the surface.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.NormalUpdate; Player.DashUpdate; Player.SuperJump; Player.OnCollideV],
    snippet: raw(block: true, lang: "cs", "// extended Super: ground refill occurs before the late Dash jump\nSpeed.X = SuperJumpH * (int) Facing;\nSpeed.Y = JumpSpeed;\n...\n// reverse down-diagonal landing\nSpeed.X *= DodgeSlideSpeedMult;\nDucking = true;\n// crouched SuperJump\nSpeed.X *= DuckSuperJumpXMult;\nSpeed.Y *= DuckSuperJumpYMult;"),
    note: [Superwave 没有独立状态：先利用地面回填完成延长 Super，再反向向下斜冲落到同一 surface，最后以蹲伏 SuperJump 的 1.25×/0.5×倍率反向起跳。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [normal_update; dash_update; super_jump; move_axis_amount]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [superwave_chains_extended_super_into_a_reverse_wavedash], note: [本地关键帧：11 帧为 260/-105 且 dash 已回填；22 帧反向落地、ducking 且水平速度低于 -200；27 帧反向 Hyper 为 -325/-52.5 且 dash 再次回填。]),
  e2e-evidence: none,
  candidate-e2e: "dash-superwave",
)
