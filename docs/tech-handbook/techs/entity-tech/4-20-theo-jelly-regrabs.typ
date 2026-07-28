#import "../../template.typ": tech, evidence

#tech(
  id: "4.20",
  title-zh: "Theo／水母重抓",
  title-en: "Theo/Jelly Regrabs",
  status: "unimplemented",
  description-zh: [冲刺中抓到 Theo 或水母会取消冲刺但保留速度；先丢再冲刺抓回，也能在已携物时完成重抓。],
  description-en: [Grabbing Theo or a jelly during a dash cancels dash state while preserving momentum; throw-and-regrab setups work while already carrying one.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Source/Holdable.cs],
    symbol: [Player.DashUpdate / Player.PickupCoroutine / Holdable.Release],
    snippet: raw(block: true, lang: "cs", "if (Holding == null && Input.Grab.Check && !IsTired && CanUnDuck)\n    if (hold.Check(this) && Pickup(hold)) return StPickup;\n...\ngravityTimer = .1f;\ncannotHoldTimer = cannotHoldDelay;"),
    note: [DashUpdate 在 DashEnd 之前扫描 Holdable；成功抓取立即进入 Pickup，0.16 秒 tween 后恢复抓取瞬间的 oldSpeed。Release 同时建立 gravity 与 CannotHold 窗口，所以丢下后必须等锁定结束再穿过物品重抓。Rust 已覆盖 Theo 路径；Jelly/Glider 尚无 runtime，条目暂不判 implemented。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [try_pickup_theo / release_theo / dash_update / pickup_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [dash_pickup_cancels_into_source_pickup_tween_and_restores_speed / theo_neutral_drop_dash_regrab_waits_out_cannot_hold]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.20-theo-regrab.ts], symbol: [entity-4.20-theo-regrab], note: [独立 MapPart 先抓 Theo，再中性放下、等待 CannotHold，并在 240 水平 Dash 中重抓；真实 Everest 尚待 FIFO 锁内采集。]),
)
