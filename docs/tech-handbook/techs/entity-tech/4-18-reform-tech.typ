#import "../../template.typ": tech, evidence

#tech(
  id: "4.18",
  title-zh: "重组方块技巧",
  title-en: "Reform Tech",
  status: "implemented",
  description-zh: [可消失并重生的方块在重组期间先后恢复实体、尖刺和附属物；玩家可利用各阶段的非实体与实体窗口。],
  description-en: [Reforming blocks restore their solid body and attached hazards in distinct phases, creating temporary overlap and interaction windows.],
  source-evidence: evidence(
    path: [Celeste/MoveBlock.cs],
    symbol: [MoveBlock.Controller],
    snippet: raw(block: true, lang: "cs", "DisableStaticMovers();\nCollidable = false;\nVisible = false;\nyield return 2.2f;\nCollidable = true;\n...\nyield return 0.6f;\nVisible = true;\nEnableStaticMovers();"),
    note: [破碎时先禁用附属 StaticMover 并同时关闭 body/visual；2.2 秒后 body 先恢复碰撞，随后 debris 回归的 0.2+0.6 秒完成后才恢复可见性与附属物，形成严格的 0.8 秒实体窗口。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [MoveBlockSnapshot / advance_move_blocks], note: [持久化 break/reform phase、Collidable 等价碰撞位置、visible 与 static_movers_enabled，保持拆分模拟可组合。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [move_block_reform_body_precedes_visibility_and_static_movers_by_point_eight]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.18-reform-tech.ts], symbol: [entity-4.18-reform-tech], note: [独立 MoveBlock、撞墙与附着尖刺场景真实观察到 break 后约 2.2 秒 body 恢复、尖刺仍禁用，再过约 0.8 秒 visual/尖刺恢复。371 个状态九字段逐帧一致，position/speed 最大误差均为 0；完整录制已生成视频与 poster。]),
  candidate-e2e: none,
)
