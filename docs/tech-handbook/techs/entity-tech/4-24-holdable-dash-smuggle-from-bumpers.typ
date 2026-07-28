#import "../../template.typ": tech, evidence

#tech(
  id: "4.24",
  title-zh: "Bumper 携物冲刺走私",
  title-en: "Holdable Dash Smuggle (From bumpers)",
  status: "unimplemented",
  description-zh: [携物撞 Bumper 的冻结帧内短暂松开并重抓，可在本来不能冲刺的携物状态中插入一次冲刺。],
  description-en: [During bumper freeze, briefly release and regrab the holdable to insert a dash into a state that normally forbids dashing while carrying.],
  source-evidence: evidence(
    path: [Source/Bumper.cs / Source/Player/Player.cs],
    symbol: [Bumper.OnPlayer / Player.ExplodeLaunch / Player.LaunchUpdate],
    snippet: raw(block: true, lang: "cs", "player.ExplodeLaunch(Position, snapUp: false);\n...\nCeleste.Freeze(0.1f);\nRefillDash();\ndashCooldownTimer = 0.2f;\nStateMachine.State = 7;"),
    note: [玩家更新先处理松抓，随后 Bumper.OnPlayer 在同帧触发 0.1 秒冻结、补充冲刺并进入 Launch；LaunchUpdate 又把 CanDash 检查放在 Holdable 抓取之前。候选审查确认第 27 帧差异发生在 Bumper→Launch 之前，不能以放宽 PickupCollider 或调整 Launch 的输入优先级伪造；因而可在碰撞帧放下物品，冻结后用缓冲冲刺离开，再穿过物品重抓的语义保持待真机验证。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [initialize_bumpers / advance_bumpers / interact / launch_update / try_pickup_theo], note: [The 2026-07-28 physical-vendor run `2026-07-28T15-09-55.721Z-76244-629d6afe-ab3c-4223-9904-f3a162191a39` authenticated its nonce and spawned PID and cleaned both owned children. Collector state 0 recorded Bumper Position `(132.48375,490.00656)`, Counter `9.262819`; state 27 recorded `(129.35115,490.28568)`, Counter `10.506892`. Rust nevertheless remained Normal at f27 while Everest entered Launch. This rejects a static map centre or missing random seed as the explanation; the remaining discrepancy is the source Bumper's two-axis SineWave rate/component function, so the technique remains candidate.]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [bumper_freeze_smuggle_releases_dashes_and_regrabs_theo / bumper_smuggle_releases_down_after_buffered_diagonal_dash_to_regrab_theo]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.24-bumper-holdable-dash-smuggle.ts], symbol: [entity-4.24-bumper-holdable-dash-smuggle], note: [最终真实候选第 27 帧先差：Rust 仍为 Normal、速度 (61.667,0)，Everest 已由 Bumper 进入 Launch、速度 (-280,-150)；最大位置／速度误差 110／358.333344，保持未实现。]),
)
