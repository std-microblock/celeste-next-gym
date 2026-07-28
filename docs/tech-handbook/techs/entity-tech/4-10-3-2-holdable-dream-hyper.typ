#import "../../template.typ": tech, evidence

#tech(
  id: "4.10.3.2",
  title-zh: "携物 Dream Hyper",
  title-en: "Holdable Dream Hyper",
  status: "implemented",
  description-zh: [完成 Dream Smuggle 并在出口抓墙后，快速丢出投掷物、向反方向 Hyper，再重新抓回投掷物。],
  description-en: [After smuggling and grabbing at the exit, throw the holdable, hyper away, and regrab it to leave with both the item and hyper speed.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Source/Holdable.cs / Source/TheoCrystal.cs],
    symbol: [Player.DreamDashEnd / Player.Throw / Holdable.Release],
    snippet: raw(block: true, lang: "cs", "Holding.Release(force);\nHolding = null;\n...\nHold.PickupCollider = new Hitbox(16f, 22f, -8f, -16f);\n...\nCannotHold = 0.1f;"),
    note: [出口抓墙保留 jump grace；投掷先把物品从玩家分离并建立 0.1 秒 CannotHold，再由反向蹲冲刺跳消费 grace，窗口结束后才能重新抓取。`TheoCrystal` 的实际抓取框是 16×22，不能为掩盖候选差异任意放宽。完整 E2E 必须依次观测 exit-grab、release、blocked regrab、Hyper 与 regrab。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [release_theo / super_jump / try_pickup_theo]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [holdable_dream_hyper_throw_cannot_hold_hyper_and_regrab_are_split_composable / holdable_dream_hyper_regrabs_on_frame_169_after_theo_release_curve]),
  e2e-evidence: evidence(
    path: [scripts/e2e-real/scenarios/playground/entity-4.10.3.2-holdable-dream-hyper.ts; .tmp/e2e-runs/2026-07-28T16-11-22.799Z-109360-323b6fd0-77c5-4fd5-8cf0-ba9e5e156a3b/manifest.json],
    symbol: [entity-4.10.3.2-holdable-dream-hyper],
    note: [2026-07-28 在受锁主工作区的物理 `vendor/celeste-game` 上运行；runner nonce 与 spawned Celeste PID 精确匹配，隔离存档／临时目录、动态 loopback ports 与受控清理均完成。240 个输入帧连同初始快照共 241 个状态，position、speed、state、facing、dashes、stamina、grounded、ducking、death 九字段逐帧一致，position 与 speed 最大误差均为 0。语义上完整观测 exit-grab、release、CannotHold 阻止立即重抓、325 Hyper，随后在原定义的 16×22 PickupCollider 上进入 Pickup 并重新抓回 Theo。],
  ),
  candidate-e2e: none,
)
