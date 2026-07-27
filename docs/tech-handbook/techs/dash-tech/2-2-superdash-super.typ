#import "../../template.typ": tech, evidence

#tech(
  id: "2.2",
  title-zh: "Super 冲刺跳",
  title-en: "Superdash (Super)",
  status: "implemented",
  description-zh: [在地面水平冲刺结束前起跳，可把冲刺转成初速约 260 的长距离跳跃；它还能延长或反向。],
  description-en: [Jumping before a grounded horizontal dash ends produces a long jump with about 260 initial horizontal speed and supports extended or reverse variants.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.DashUpdate; Player.SuperJump],
    snippet: raw(block: true, lang: "cs", "if (DashDir.Y == 0 && CanUnDuck && Input.Jump.Pressed && jumpGraceTimer > 0) {\n    SuperJump();\n    return StNormal;\n}\n...\nSpeed.X = SuperJumpH * (int) Facing;\nSpeed.Y = JumpSpeed;"),
    note: [水平冲刺、土狼时间与跳跃输入共同进入 SuperJump，设置 260/-105。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [dash_update; super_jump]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [superdash_sets_source_launch_speed_and_spends_dash]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [super], note: [真实关键帧速度 260/-105，九类字段 max speed error 0。]),
  candidate-e2e: none,
)
