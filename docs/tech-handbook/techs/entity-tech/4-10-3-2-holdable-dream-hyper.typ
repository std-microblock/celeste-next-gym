#import "../../template.typ": tech, evidence

#tech(
  id: "4.10.3.2",
  title-zh: "携物 Dream Hyper",
  title-en: "Holdable Dream Hyper",
  status: "unimplemented",
  description-zh: [完成 Dream Smuggle 并在出口抓墙后，快速丢出投掷物、向反方向 Hyper，再重新抓回投掷物。],
  description-en: [After smuggling and grabbing at the exit, throw the holdable, hyper away, and regrab it to leave with both the item and hyper speed.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Source/Holdable.cs],
    symbol: [Player.DreamDashEnd / Player.Throw / Holdable.Release],
    snippet: raw(block: true, lang: "cs", "Holding.Release(force);\nHolding = null;\n...\nCannotHold = 0.1f;"),
    note: [出口抓墙保留 jump grace；投掷先把物品从玩家分离并建立 0.1 秒 CannotHold，再由反向蹲冲刺跳消费 grace，窗口结束后才能重新抓取。完整 E2E 必须依次观测 exit-grab、release、blocked regrab、Hyper 与 regrab。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [release_theo / super_jump / try_pickup_theo]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [holdable_dream_hyper_throw_cannot_hold_hyper_and_regrab_are_split_composable]),
  e2e-evidence: none,
  candidate-e2e: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [entity-4.10.3.2-holdable-dream-hyper],
    note: [延长后的最终候选已完整形成出口抓持、松物、CannotHold、325 Hyper 与重新抓回 Theo；但第 169 帧先差：Rust 在 (370,496) 仍以 -90 移动，Everest 在 (371,496) 已进入 Pickup 且速度为零，最大位置／速度误差 3／90，故保持未实现。],
  ),
)
