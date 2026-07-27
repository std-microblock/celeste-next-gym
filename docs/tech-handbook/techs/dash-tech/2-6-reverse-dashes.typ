#import "../../template.typ": tech, evidence

#tech(
  id: "2.6",
  title-zh: "反向冲刺技巧",
  title-en: "Reverse Dashes",
  status: "implemented",
  description-zh: [Super、Hyper 和 Wavedash 的最终方向由起跳时的水平输入决定，因此可以朝与冲刺相反的方向跳出。],
  description-en: [The jump direction, not the dash direction, determines the exit of supers, hypers, and wavedashes, allowing reverse variants.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.Update; Player.SuperJump],
    snippet: raw(block: true, lang: "cs", "if (moveX != 0 && InControl && StateMachine.State != StClimb)\n    Facing = (Facings) moveX;\n...\nSpeed.X = SuperJumpH * (int) Facing;\nSpeed.Y = JumpSpeed;"),
    note: [状态回调前的水平输入更新 Facing，SuperJump 用 Facing 而不是冲刺方向决定水平出口。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [step; super_jump]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [reverse_super_uses_jump_frame_facing_not_dash_direction]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [reverse-super], note: [向右冲刺后向左跳出，关键帧 facing=Left、速度 -260/-105；max speed error 0.000001。]),
  candidate-e2e: none,
)
