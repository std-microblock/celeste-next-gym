#import "../../template.typ": tech, evidence

#tech(
  id: "4.23",
  title-zh: "Theo／水母 Ultra",
  title-en: "Theo/Jelly Ultras",
  status: "unimplemented",
  description-zh: [在贴地 Ultra 中重抓 Theo 或水母，可提前取消冲刺结束逻辑并保留倍增后的高速。],
  description-en: [Regrabbing Theo or a jelly interrupts a grounded ultra before dash-end speed cleanup, preserving the multiplied velocity.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Source/Holdable.cs / Source/Glider.cs],
    symbol: [Player.DashUpdate / Player.PickupCoroutine / Holdable.Check],
    snippet: raw(block: true, lang: "cs", "if (component.Check(this) && Pickup(component))\n    return 8;\n...\nVector2 oldSpeed = Speed;\nSpeed = Vector2.Zero;\n...\nSpeed = oldSpeed;"),
    note: [DashUpdate 在冲刺自然结束前允许抓取任意 Holdable；Pickup 的 0.16 秒协程先缓存速度、归零，再恢复 oldSpeed。Theo 与 Glider 均已接入同一 Rust 抓取顺序并保留 360 Ultra 速度；Jelly 真机对照与视频未完成前仍保持未实现。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [dash_update / try_pickup_holdable / pickup_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [grounded_ultra_pickup_cancel_skips_dash_end_speed_normalization / grounded_ultra_glider_pickup_cancel_preserves_multiplied_speed]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.23-theo-ultra.ts / scripts/e2e-real/scenarios/playground/entity-4.23-jelly-ultra.ts], symbol: [entity-4.23-theo-ultra / entity-4.23-jelly-ultra], note: [Theo 与 Jelly 各有独立贴地 Ultra MapPart，并要求 Pickup 后恢复 360 水平速度；真实 Everest 尚待锁内采集。]),
)
