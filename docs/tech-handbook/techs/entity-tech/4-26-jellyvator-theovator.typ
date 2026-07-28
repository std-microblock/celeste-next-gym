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
    note: [中性放下先建立 CannotHold 窗口；窗口结束后，上冲刺中的 DashUpdate 可抓取物品并进入 Pickup。协程缓存抓取瞬间的 -240 向上 Dash 速度，0.16 秒后恢复。Theo 与 Glider 的不同锁定时长均已进入 Rust 回归，真实双实体 E2E 未完成前不计覆盖。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [release_theo / release_glider / dash_update / pickup_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [dash_pickup_runs_before_the_dash_coroutine_samples_direction / dash_pickup_after_the_initial_yield_caches_live_updash_speed / theovator_regrabs_after_updash_speed_is_live_and_restores_it_after_pickup / jellyvator_regrabs_updash_and_restores_vertical_speed], note: [DashUpdate 的 Holdable 检查先于 DashCoroutine 初始 yield 后的 DashDir 赋值；初始 yield 结束后的下一帧才发布 -240 上冲速度，随后 Theo/Jelly 重抓会缓存并在 Pickup tween 后恢复该速度。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.26-theovator.ts / scripts/e2e-real/scenarios/playground/entity-4.26-jellyvator.ts], symbol: [entity-4.26-theovator / entity-4.26-jellyvator], note: [Jelly 真实场景已完成重抓上冲语义，但第 47 帧游戏位于 (60, 492)、速度 (0, -240)、状态 Dash，Rust 仍位于 (60, 496)、速度为零、状态 Pickup；最大位置／速度误差为 46／240，故双变体保留 candidate。]),
)
