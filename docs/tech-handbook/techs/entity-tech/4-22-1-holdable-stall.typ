#import "../../template.typ": tech, evidence

#tech(
  id: "4.22.1",
  title-zh: "携物滞空",
  title-en: "Holdable Stall",
  status: "unimplemented",
  description-zh: [有少量纵向速度时反复中性放下并快速重抓，可延长空中停留时间；两只水母可把它扩展成梯子。],
  description-en: [Repeated neutral drops and quick regrabs extend airtime, and two jellies can turn the stall into laddering.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Source/Holdable.cs],
    symbol: [Player.Drop / Player.PickupCoroutine / Holdable.Release],
    snippet: raw(block: true, lang: "cs", "Speed = Vector2.Zero;\nyield return tween.Wait();\nSpeed = oldSpeed;\nSpeed.Y = Math.Min(Speed.Y, 0);"),
    note: [每次重抓进入 0.16 秒 Pickup tween，期间玩家速度归零；结束时只恢复非向下的旧纵速。Glider 中性放下建立 0.3 秒 CannotHold，玩家仍受 0.35 秒最短持有时间限制；Rust 已证明 tween 滞空与再次抓取原语，真实重复链尚待验收。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [release_glider / try_pickup_glider / pickup_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [glider_pickup_tween_stalls_then_restores_only_upward_speed / released_glider_obeys_long_lockout_then_can_be_regrabbed]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.22.1-holdable-stall.ts], symbol: [entity-4.22.1-holdable-stall], note: [独立空中 Jelly MapPart 尝试两轮中性放下与 Pickup tween；真实 Everest 尚待 FIFO 锁内调时并录制。]),
)
