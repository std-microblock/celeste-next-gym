#import "../../template.typ": tech, evidence

#tech(
  id: "4.3",
  title-zh: "Bumper 穿越",
  title-en: "Bumper Clip",
  status: "implemented",
  description-zh: [Bumper 被触发后碰撞箱会短暂消失；在冻结帧内缓冲冲刺并选对角度，可以直接穿过它。],
  description-en: [A hit bumper briefly loses its collider, allowing a correctly angled buffered dash to pass through during the opening.],
  source-evidence: evidence(
    path: [Source/Bumper.cs / Source/Player/Player.cs],
    symbol: [Bumper.OnPlayer / Player.ExplodeLaunch],
    snippet: raw(block: true, lang: "cs", "else if (respawnTimer <= 0f) {\n    respawnTimer = 0.6f;\n    player.ExplodeLaunch(Position, snapUp: false);\n}\n...\nCeleste.Freeze(0.1f);"),
    note: [Bumper 只在 `respawnTimer <= 0` 时再次触发，命中后先写入 0.6 秒冷却；它不是 Solid，因此首次爆炸的 0.1 秒冻结结束后，缓冲的水平冲刺可以在冷却仍有效时穿过且不会再次被发射。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [interact / explode_launch], note: [快照保存 Bumper 的 0.6 秒复用计时；冷却期间仍参与场景但跳过再次 launch，Player 的冲刺移动继续穿过实体区域。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [bumper_clip_dashes_back_through_during_the_point_six_second_reuse_window]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [entity-4.3-bumper-clip], note: [真实 Bumper 首次把玩家送入 Launch；输入帧 20 缓冲冲刺，状态帧 25 获得 240 水平速度，状态帧 28 已穿至 x=604 且未再次发射。51 个状态九类字段逐帧一致，最大 position 误差 0、speed 误差 0.000015。]),
  candidate-e2e: none,
)
