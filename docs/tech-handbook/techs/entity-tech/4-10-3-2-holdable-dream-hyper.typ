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
    note: [修正后的真实候选已形成出口抓持 `grab=49`、松物 `release=54`、CannotHold 阻止立即回抓与 325 Hyper `hyper=63`；最终 `regrab=false`，因此完整链仍缺重新抓回 Theo，保持未实现。],
  ),
)
