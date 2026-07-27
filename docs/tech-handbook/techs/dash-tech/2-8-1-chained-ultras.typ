#import "../../template.typ": tech, evidence

#tech(
  id: "2.8.1",
  title-zh: "连续 Ultra",
  title-en: "Chained Ultras",
  status: "implemented",
  description-zh: [在合适地形上反复衔接 Ultra，使每次 1.2 倍加速继续乘算，从而快速累积极高速度。],
  description-en: [Suitable terrain can chain multiple ultras so their 1.2 multipliers compound into very high speed.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.OnCollideV],
    snippet: raw(block: true, lang: "cs", "Speed.Y = 0;\nSpeed.X *= DodgeSlideSpeedMult;\nDucking = true;"),
    note: [每次符合条件的落地都会重新对当时的水平速度应用 1.2 倍；源码没有一次性标记，因此多个落地倍率自然连乘。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [move_axis_amount]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [chained_ultras_compound_two_landing_multipliers], note: [本地语义帧：第一次倍率后 speed.x≈193.24673，第二次倍率后 speed.x≈231.89609。]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [dash-chained-ultras; verifyChainedUltras], note: [真实 41 帧场景观察到两次独立倍率：第一次落地 speed.x=193.24673，第二次贴地 Ultra 得到 231.89609，精确为前者的 1.2 倍；九类核心字段最大位置与速度误差均为 0。]),
  candidate-e2e: none,
)
