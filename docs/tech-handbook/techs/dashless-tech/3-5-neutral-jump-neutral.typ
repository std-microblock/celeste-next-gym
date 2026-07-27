#import "../../template.typ": tech, evidence

#tech(
  id: "3.5",
  title-zh: "Neutral 中性墙跳",
  title-en: "Neutral Jump (Neutral)",
  status: "unimplemented",
  description-zh: [不按方向离墙起跳，再立即按回墙方向，可不消耗体力地反复攀升直墙或不规则墙面。],
  description-en: [Jump away from a wall with no direction held, then steer back toward it to climb repeatedly without stamina.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.NormalUpdate; Player.WallJumpCheck; Player.WallJump],
    snippet: raw(block: true, lang: "cs", "private const int WallJumpCheckDist = 3;\n...\nif (canUnduck && WallJumpCheck(1))\n    WallJump(-1);\n...\nif (moveX != 0) {\n    forceMoveX = dir;\n    forceMoveXTimer = WallJumpForceTime;\n}\nSpeed.X = WallJumpHSpeed * dir;\nSpeed.Y = JumpSpeed;"),
    note: [NormalUpdate 先用 3 像素探针寻找墙面。无方向输入时 WallJump 不设置 0.16 秒强制离墙计时，因此下一帧即可转回墙面；墙跳本身设置 130 水平速度与 -105 垂直速度且不消耗体力。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [WALL_JUMP_CHECK_DIST; normal_update; wall_jump_check],
    note: [模拟器按源码的左右探测顺序执行 3 像素墙跳检查，并仅在跳跃帧存在水平输入时设置 force_move_x_timer。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [neutral_wall_jumps_return_for_a_second_stamina_free_jump],
    note: [标准直墙上第 0 帧无方向墙跳，第 1 帧起转回墙面，并在第 26 帧再次无方向墙跳；两次跳跃均保持体力与零强制输入计时，并核对九类核心状态字段。],
  ),
  e2e-evidence: none,
  candidate-e2e: "neutral-jump",
)
