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
    note: [每次重抓进入 0.16 秒 Pickup tween，期间玩家速度归零；结束时只恢复非向下的旧纵速。中性放下的 0.1 秒 CannotHold 与玩家 0.35 秒最短持有时间共同决定可重复节奏。Rust 有 Theo 原语，但尚无重复滞空链或双 Jelly 证明。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [release_theo / try_pickup_theo / pickup_update]),
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
