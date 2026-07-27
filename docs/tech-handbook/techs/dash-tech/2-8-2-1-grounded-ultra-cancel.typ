#import "../../template.typ": tech, evidence

#tech(
  id: "2.8.2.1",
  title-zh: "贴地 Ultra 取消",
  title-en: "Grounded Ultra Cancel",
  status: "unimplemented",
  description-zh: [用抓取投掷物、跳过过场或弹跳等方式提前打断贴地 Ultra，可以绕过冲刺结束时的速度重置。],
  description-en: [Interrupting a grounded ultra before dash end, for example with a grab or bounce, preserves speed that the normal dash exit would remove.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.DashCoroutine; Player.DashUpdate; Player.DashEnd; Player.PickupCoroutine],
    snippet: raw(block: true, lang: "cs", "if (onGround && DashDir.X != 0 && DashDir.Y > 0 && Speed.Y > 0) {\n    Speed.Y = 0; Speed.X *= 1.2f; Ducking = true;\n}\n...\nif (Holding == null && DashDir != Vector2.Zero && Input.Grab.Check && CanUnDuck)\n    if (hold.Check(this) && Pickup(hold)) return StPickup;\n...\nyield return .15f;\nif (DashDir.Y <= 0) Speed = DashDir * 160f;"),
    note: [Grounded Ultra 先把保留的 300 水平速度乘 1.2 并切为蹲伏碰撞箱。后续 DashUpdate 在跳跃分支和自然结束前检查 Holdable，且 CanUnDuck 必须先通过；切到 Pickup 会执行 DashEnd 并终止原 Dash coroutine，因此 0.15 秒后的 160 速度归一化不会发生，PickupCoroutine 在 0.16 秒后恢复缓存的 360。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/types.rs], symbol: [dash_update; try_pickup_theo; pickup_update; PlayerSnapshot.pickup_old_speed]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [grounded_ultra_pickup_cancel_skips_dash_end_speed_normalization]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/dash-grounded-ultra-cancel.ts; scripts/e2e-real/scenarios/playground/dash-grounded-ultra-cancel-control.ts], symbol: [dash-grounded-ultra-cancel; dash-grounded-ultra-cancel-control; verifyGroundedUltraCancel], note: [同一独立 Theo MapPart 上的取消与自然 DashEnd 对照候选；真实九字段 E2E 未完成前保持 unimplemented。]),
)
