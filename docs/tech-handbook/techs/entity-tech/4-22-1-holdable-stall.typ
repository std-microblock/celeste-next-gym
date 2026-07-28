#import "../../template.typ": tech, evidence

#tech(
  id: "4.22.1",
  title-zh: "携物滞空",
  title-en: "Holdable Stall",
  status: "implemented",
  description-zh: [有少量纵向速度时反复中性放下并快速重抓，可延长空中停留时间；两只水母可把它扩展成梯子。],
  description-en: [Repeated neutral drops and quick regrabs extend airtime, and two jellies can turn the stall into laddering.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Source/Holdable.cs],
    symbol: [Player.Drop / Player.PickupCoroutine / Holdable.Release],
    snippet: raw(block: true, lang: "cs", "Speed = Vector2.Zero;\nyield return tween.Wait();\nSpeed = oldSpeed;\nSpeed.Y = Math.Min(Speed.Y, 0);"),
    note: [每次重抓进入 0.16 秒 Pickup tween，期间玩家速度归零；结束时只恢复非向下的旧纵速。Glider 中性放下建立 0.3 秒 CannotHold，玩家仍受 0.35 秒最短持有时间限制；Rust 已证明 tween 滞空与再次抓取原语，真实重复链尚待验收。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [release_glider / try_pickup_glider / pickup_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [glider_pickup_tween_stalls_then_clamps_upward_speed / released_glider_obeys_long_lockout_then_can_be_regrabbed]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.22.1-holdable-stall.ts], symbol: [entity-4.22.1-holdable-stall], note: [独立空中双 Jelly MapPart 完成两轮 Pickup 与中性放下；Player.cs 的 SlowFall 分支使首轮 tween 后把 -20 上升速度钳到 -105。97 个真实状态的 position／speed 最大误差均为 0，state、facing、dashes、stamina、grounded、ducking、death 逐帧一致。]),
  candidate-e2e: none,
)
