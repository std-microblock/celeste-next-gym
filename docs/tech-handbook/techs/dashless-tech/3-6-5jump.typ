#import "../../template.typ": tech, evidence

#tech(
  id: "3.6",
  title-zh: "5 格跳",
  title-en: "5jump",
  status: "unimplemented",
  description-zh: [用 neutral 攀跳到墙顶，再从顶部追加跳跃或攀跳，可跨越约 5 格宽的缺口。],
  description-en: [A neutral climb jump reaches the wall top, followed by another jump from the lip to cross a five-tile gap.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.ClimbUpdate; Player.ClimbJump; Player.Jump],
    snippet: raw(block: true, lang: "cs", "if (Input.Jump.Pressed && (!Ducking || CanUnDuck)) {\n    if (moveX == -(int)Facing) WallJump(-(int)Facing);\n    else ClimbJump();\n    return StNormal;\n}\n...\nprivate void ClimbJump() {\n    if (!onGround) Stamina -= ClimbJumpCost;\n    Jump(false, false);\n}\n...\nSpeed.X += JumpHBoost * moveX;\nSpeed.Y = JumpSpeed;"),
    note: [攀爬状态可以先用无方向 ClimbJump 保持贴近墙面；到达墙唇时再次 ClimbJump，Jump 会把当帧方向乘以 40 加到水平速度，并重新获得 -105 垂直速度。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [climb_update; climb_jump; JUMP_H_BOOST],
    note: [模拟器复用源码顺序的 climb_jump，在每次空中攀跳扣除 27.5 体力，并保留两次攀跳之间的方向、变高跳与墙面探测。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [five_jump_chains_neutral_and_lip_climb_jumps_across_five_tiles],
    note: [独立场景先 neutral 攀跳、再在墙唇执行第二次攀跳，验证两次 -27.5 体力消耗，并落到从墙角相隔 5 格的目标平台。],
  ),
  e2e-evidence: none,
  candidate-e2e: "five-jump",
)
