#import "../../template.typ": tech, evidence

#tech(
  id: "3.11",
  title-zh: "体力取消",
  title-en: "Stamina Cancel",
  status: "implemented",
  description-zh: [攀墙时快速点按抓取而非持续抓取，可改变攀爬和滑落节奏，从而用更少体力获得相同高度。],
  description-en: [Tapping grab while climbing changes the climb cycle and can cover the same height with less stamina than holding grab continuously.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.ClimbBegin; Player.ClimbUpdate],
    snippet: raw(block: true, lang: "cs", "climbNoMoveTimer = ClimbNoMoveTime;\n...\nlastClimbMove = Math.Sign(target);\n...\nif (climbNoMoveTimer <= 0) {\n    if (lastClimbMove == -1)\n        Stamina -= ClimbUpCost * Engine.DeltaTime;\n    else if (lastClimbMove == 0)\n        Stamina -= ClimbStillCost * Engine.DeltaTime;\n}"),
    note: [每次重新抓墙都会重置 0.1 秒免耗窗口。窗口结束后才按上爬或静止扣体力；松抓回到 Normal 后可以再次进入 ClimbBegin。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [normal_update; climb_update; PlayerSnapshot.climb_no_move_timer]),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [stamina_cancel_regrabs_to_reset_the_no_move_cost_window; climbing_down_does_not_pay_the_stationary_stamina_cost],
    note: [回归既验证松抓重抓会重置免耗窗口，也验证体力扣费取决于源码的实际 target 方向：向下攀爬不误扣静止体力。],
  ),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [stamina-cancel], note: [真实攀墙中松抓、速度回落并重抓；31 个状态帧的九类核心字段逐帧一致，max position error 0，max speed error 0。]),
  candidate-e2e: none,
)
