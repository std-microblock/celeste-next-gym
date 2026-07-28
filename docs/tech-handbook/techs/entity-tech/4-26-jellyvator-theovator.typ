#import "../../template.typ": tech, evidence

#tech(
  id: "4.26",
  title-zh: "Jellyvator／Theovator",
  title-en: "Jellyvator / Theovator",
  status: "unimplemented",
  description-zh: [中性放下投掷物后向上冲刺并重新抓取，抓取会取消冲刺但保留向上动量，从而获得额外高度。],
  description-en: [Neutral-drop a throwable, up-dash into it, and regrab to cancel dash state while retaining upward momentum.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Source/Holdable.cs],
    symbol: [Player.DashUpdate / Player.PickupCoroutine / Holdable.Release],
    snippet: raw(block: true, lang: "cs", "if (component.Check(this) && Pickup(component))\n    return 8;\n...\nVector2 oldSpeed = Speed;\nSpeed = Vector2.Zero;\n...\nSpeed = oldSpeed;\nSpeed.Y = Math.Min(Speed.Y, 0f);"),
    note: [中性放下先建立 CannotHold 窗口；窗口结束后，上冲刺中的 DashUpdate 可抓取物品并进入 Pickup。协程缓存抓取瞬间的向上 Dash 速度，0.16 秒后恢复且只把向下速度钳到零。Theo 路径已实现；Jelly/Glider runtime 与真实实体 E2E 缺失，所以不计覆盖。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [release_theo / dash_update / pickup_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [theovator_regrabs_after_updash_speed_is_live_and_restores_it_after_pickup]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.26-theovator.ts], symbol: [entity-4.26-theovator], note: [独立 MapPart 验证 Theo 中性放下、上冲刺重抓与恢复 -240 Y 速度；仅为 Theovator 候选，不能证明 Jellyvator。]),
)
