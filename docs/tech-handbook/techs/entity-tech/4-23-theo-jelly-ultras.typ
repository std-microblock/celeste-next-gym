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
    note: [DashUpdate 在冲刺自然结束前允许抓取任意 Holdable；Pickup 的 0.16 秒协程先缓存速度、归零，再恢复 oldSpeed。Theo 路径已在 Rust 和真实对照中成立，但本条明确同时要求 Jelly/Glider；模拟器尚无 Glider runtime 与第二个真实实体 E2E，因此保持未实现。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [dash_update / try_pickup_theo / pickup_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [grounded_ultra_pickup_cancel_skips_dash_end_speed_normalization]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.23-theo-ultra.ts], symbol: [entity-4.23-theo-ultra], note: [独立 MapPart 验证 Theo 抓取打断贴地 Ultra 并恢复 360 水平速度；只覆盖 Theo，不能替代缺失的 Jelly/Glider runtime 与真实 E2E。]),
)
