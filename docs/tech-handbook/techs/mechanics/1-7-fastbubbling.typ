#import "../../template.typ": tech, evidence

#tech(
  id: "1.7",
  title-zh: "快速泡泡",
  title-en: "Fastbubbling",
  status: "implemented",
  description-zh: [进入静止的绿泡或红泡后立即按住冲刺，可以让泡泡立刻按输入方向启动，而且不消耗玩家冲刺次数。],
  description-en: [Holding dash as a stationary green or red bubble is entered launches it immediately in the chosen direction without spending a dash.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.StartDash; Player.BoostUpdate],
    snippet: raw(block: true, lang: "cs", "public int StartDash() {\n    Dashes = Math.Max(0, Dashes - 1);\n    Input.Dash.ConsumeBuffer();\n    return StDash;\n}\n...\nif (Input.Dash.Pressed) {\n    Input.Dash.ConsumePress();\n    return boostRed ? StRedDash : StDash;\n}"),
    note: [普通 StartDash 会显式扣除一次冲刺；BoostUpdate 检测到按键后直接切换到 Dash/RedDash，没有调用 StartDash，因此泡泡可立即启动且不消耗 Dashes。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [boost_update; begin_dash]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [fastbubble_manual_dash_releases_immediately_without_spending_dash]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [playground-green-booster-right], note: [真实第 1 帧进入 Boost，第 2 帧按冲刺立即进入 Dash 且 dashes 仍为 1，第 6 帧达到 240 水平速度。25 个状态帧九类字段逐帧一致，max position error 0，max speed error 0.000011。]),
  candidate-e2e: none,
)
