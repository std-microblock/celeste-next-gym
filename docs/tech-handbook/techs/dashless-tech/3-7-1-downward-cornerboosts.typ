#import "../../template.typ": tech, evidence

#tech(
  id: "3.7.1",
  title-zh: "向下 Cornerboost",
  title-en: "Downward Cornerboosts",
  status: "implemented",
  description-zh: [通过保持向上速度、冲刺状态、较远墙距或低体力等条件避免真正抓墙，可在向下经过墙角时保留速度并完成 Cornerboost。],
  description-en: [A downward cornerboost avoids entering the grab state through movement, dash, spacing, or stamina conditions so speed is not erased.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.NormalUpdate; Player.ClimbCheck; Player.WallJumpCheck; Player.NormalEnd],
    snippet: raw(block: true, lang: "cs", "if (Speed.Y >= 0 && Math.Sign(Speed.X) != -(int)Facing) {\n    if (ClimbCheck((int)Facing))\n        return StClimb;\n}\n...\nif (canUnduck && WallJumpCheck(1)) {\n    if (Facing == Facings.Right && Input.Grab.Check && Stamina > 0)\n        ClimbJump();\n}\n...\nprivate void NormalEnd() {\n    wallSpeedRetentionTimer = 0;\n}"),
    note: [向下移动时抓墙检查先于跳跃；真正进入 Climb 会由 NormalEnd 清除 retained speed。利用 ClimbCheck 的 2px 距离小于 WallJumpCheck 的 3px 距离，可让抓墙失败而攀跳仍命中。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [normal_update; climb_check; wall_jump_check; climb_jump],
    note: [模拟器在实际进入 Climb 时清除 retention，但允许正 Y 速度下 2px 攀抓失败、3px 墙跳成功的 Normal-state ClimbJump。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [downward_cornerboost_uses_wall_jump_probe_without_entering_climb],
    note: [独立场景以 +30 垂直速度和 160 水平速度下降，明确断言 2px climb_check 失败、3px wall_jump_check 命中；结果保持 Normal、扣除 27.5 体力并保存 195.66666。],
  ),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [downward-cornerboost],
    note: [真实游戏共 13 个状态帧：从 +30 垂直速度下降，以 2px 抓墙 miss／3px 墙跳 hit 保持 Normal；首帧保存 195.66666，第 5 状态帧返还为 191.33331。九类核心字段逐帧一致，最大位置误差 0、速度误差 0.000001。],
  ),
  candidate-e2e: none,
)
