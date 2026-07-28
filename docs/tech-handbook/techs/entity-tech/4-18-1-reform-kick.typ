#import "../../template.typ": tech, evidence

#tech(
  id: "4.18.1",
  title-zh: "Reform Kick 重组墙跳",
  title-en: "Reform Kick",
  status: "implemented",
  description-zh: [站在尚未实体化的重组方块内部，待其应开始恢复时横向离开并按跳，可从新出现的侧面墙跳。],
  description-en: [Exit a reforming non-solid block horizontally while jumping as it becomes solid to wallkick from the reappearing side.],
  source-evidence: evidence(
    path: [Celeste/MoveBlock.cs; Source/Player/Player.cs],
    symbol: [MoveBlock.Controller; Player.NormalUpdate; Player.WallJump],
    snippet: raw(block: true, lang: "cs", "yield return 2.2f;\nCollidable = true;\n...\nelse if (WallJumpCheck(1))\n    WallJump(-1);\n...\nSpeed.X = 130f * dir;\nSpeed.Y = -105f;"),
    note: [MoveBlock 在仍不可见时先恢复 Collidable；玩家已离开地面且侧面 probe 命中新 body 时，NormalUpdate 分派到普通 WallJump，立即写入远离方块的 130 水平速度与 -105 纵向速度。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [advance_move_blocks / normal_update / wall_jump]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [reform_kick_wall_jumps_from_the_newly_collidable_invisible_body]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.18.1-reform-kick.ts], symbol: [entity-4.18.1-reform-kick], note: [玩家先离开等待平台，frame 302 观察到 invisible body 恢复，并在空中从其左侧 wallkick。361 个真实状态 position 最大误差 0、speed 最大误差 0.000015，其余七类核心字段逐帧一致；完整 trace 已录制为 MP4/poster。]),
  candidate-e2e: none,
)
