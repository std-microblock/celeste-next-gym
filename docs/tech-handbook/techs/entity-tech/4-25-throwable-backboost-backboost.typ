#import "../../template.typ": tech, evidence

#tech(
  id: "4.25",
  title-zh: "投掷物 Backboost",
  title-en: "Throwable Backboost (Backboost)",
  status: "implemented",
  description-zh: [向后丢出 Theo 或水母会给玩家相反方向约 80 速度，因此短暂转身后后抛可为前进方向加速。],
  description-en: [Throwing Theo or a jelly backward grants roughly 80 speed in the opposite direction, boosting forward travel after a quick turnaround.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Source/TheoCrystal.cs / Source/Glider.cs],
    symbol: [Player.Throw / TheoCrystal.OnRelease / Glider.OnRelease],
    snippet: raw(block: true, lang: "cs", "Holding.Release(Vector2.UnitX * (float)Facing);\nSpeed.X += 80f * (float)(0 - Facing);\n...\nSpeed = force * 200f;"),
    note: [非下方向松抓调用 Throw：物品沿 Facing 释放，而玩家同帧得到反向固定 80 水平速度。Theo 与 Glider 的 OnRelease 只改变物品自身倍率（200 与 100，Glider 还把 Y 力减半），玩家 recoil 常量与顺序相同。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [release_theo]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [throwable_backboost_adds_eighty_opposite_the_throw_facing]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.25-throwable-backboost.ts], symbol: [entity-4.25-throwable-backboost], note: [独立 Theo MapPart 先抓取、短暂转身再投掷；真实 Everest 共 43 个状态，position、speed、state、facing、dashes、stamina、grounded、ducking、death 全部逐帧一致，最大 position／speed 误差均为 0。]),
  candidate-e2e: none,
)
