#import "../../template.typ": tech, evidence

#tech(
  id: "3.7.8",
  title-zh: "反向 Cornerboost",
  title-en: "Reverse Cornerboost (rcb)",
  status: "implemented",
  description-zh: [对身后的墙角攀跳可取消冲刺并保留大部分原动量，但反向跳跃加速会从速度中扣除约 40。],
  description-en: [Climb-jumping a corner behind the player cancels the dash and preserves momentum, minus the jump acceleration applied backward.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.ClimbJump; Player.Jump; Player.OnCollideH],
    snippet: raw(block: true, lang: "cs", "private void ClimbJump() {\n    if (!onGround)\n        Stamina -= ClimbJumpCost;\n    Jump(false, false);\n}\n...\nSpeed.X += JumpHBoost * moveX;\nSpeed.Y = JumpSpeed;"),
    note: [ClimbJump 先扣 27.5 体力再调用 Jump；Jump 按当前反向输入把 40 水平加速加到原速度，并设纵速 -105。角碰撞保留碰撞前水平速度，因此反向 Cornerboost 保住主体动量但减去这 40。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [climb_jump; move_axis_amount]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [reverse_cornerboost_preserves_forward_momentum_minus_backward_jump_boost]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/reverse-cornerboost.ts], symbol: [reverse-cornerboost; verifyReverseCornerboost], note: [独立墙角 MapPart 记录 21 个真实状态；state 1 保持 Normal、面向左、体力 82.5，速度为 109.16664/-105，且全程存活。九类核心字段逐帧一致，最大位置与速度误差均为 0。]),
  candidate-e2e: none,
)
