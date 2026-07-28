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
    snippet: raw(block: true, lang: "cs", "// DashUpdate\nif (hold.Check(this) && Pickup(hold)) return StPickup;\n// DashCoroutine\nyield return null;\nSpeed = lastAim * DashSpeed;\nDashDir = lastAim;\n// PickupCoroutine\nVector2 oldSpeed = Speed;\nSpeed = Vector2.Zero;\nyield return tween.Wait();\nSpeed = oldSpeed;\nSpeed.Y = Math.Min(Speed.Y, 0f);"),
    note: [DashUpdate 与 DashCoroutine 的首个 yield 并存：首个解冻的 DashUpdate 仍属于 coroutine 的 yield 帧，随后 coroutine 才发布 -240 与 DashDir；下一帧 Holdable 检查才可缓存该 live speed。Pickup 的 0.16 秒 tween 结束时恢复并向上钳制旧纵速。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [release_theo / release_glider / dash_update / pickup_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [dash_pickup_after_the_initial_yield_caches_live_updash_speed / jellyvator_regrabs_updash_and_restores_vertical_speed], note: [回归以 Grab 已持续按下为条件：第 47 帧仍为 Dash、速度 (0,-240)，第 48 帧才进入 Pickup 并缓存 -240；这锁定了 DashCoroutine 初始 yield 与 Holdable 检查的帧序。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.26-theovator.ts / scripts/e2e-real/scenarios/playground/entity-4.26-jellyvator.ts], symbol: [entity-4.26-theovator / entity-4.26-jellyvator], note: [2026-07-28 的真实首差为 Jellyvator 第 47 帧：Everest (60,492)、(0,-240)、Dash，旧 Rust (60,496)、零速、Pickup。后续修复将首个 DashCoroutine 解冻帧排除出抓取窗口，并以同一帧本地回归锁定；尚需在共享真实 E2E 环境复跑九字段比较，故保持 candidate。]),
)
