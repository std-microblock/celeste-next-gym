#import "../../template.typ": tech, evidence

#tech(
  id: "4.24",
  title-zh: "Bumper 携物冲刺走私",
  title-en: "Holdable Dash Smuggle (From bumpers)",
  status: "implemented",
  description-zh: [携物撞 Bumper 的冻结帧内短暂松开并重抓，可在本来不能冲刺的携物状态中插入一次冲刺。],
  description-en: [During bumper freeze, briefly release and regrab the holdable to insert a dash into a state that normally forbids dashing while carrying.],
  source-evidence: evidence(
    path: [Source/Bumper.cs / Source/Player/Player.cs],
    symbol: [Bumper.OnPlayer / Player.ExplodeLaunch / Player.LaunchUpdate],
    snippet: raw(block: true, lang: "cs", "player.ExplodeLaunch(Position, snapUp: false);\n...\nCeleste.Freeze(0.1f);\nRefillDash();\ndashCooldownTimer = 0.2f;\nStateMachine.State = 7;"),
    note: [玩家更新先处理松抓，随后 Bumper.OnPlayer 在同帧触发 0.1 秒冻结、补充冲刺并进入 Launch；LaunchUpdate 又把 CanDash 检查放在 Holdable 抓取之前。候选审查确认第 27 帧差异发生在 Bumper→Launch 之前，不能以放宽 PickupCollider 或调整 Launch 的输入优先级伪造；因而可在碰撞帧放下物品，冻结后用缓冲冲刺离开，再穿过物品重抓的语义保持待真机验证。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [initialize_bumpers / advance_bumpers / interact / launch_update / try_pickup_theo], note: [Bumper advances its randomized SineWave and publishes `Position` before its PlayerCollider invokes `OnPlayer`. The physical trace records f85 `(132.59096,492.19797)` and f86 `(132.72580,492.24374)`; f86's non-horizontal launch is therefore computed from the newly published f86 target, then receives the matching-direction 1.2x horizontal boost. `Player.LaunchUpdate` then scans Holdables without a `Holding == null` guard: when Grab survives the freeze, the held Theo overlaps its own pickup collider and restarts PickupCoroutine at f93. The regression locks f91–f95's Launch-to-Pickup boundary without changing collision radius or tolerance.]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [bumper_freeze_smuggle_releases_dashes_and_regrabs_theo / bumper_smuggle_releases_down_after_buffered_diagonal_dash_to_regrab_theo]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.24-bumper-holdable-dash-smuggle.ts; .tmp/e2e-runs/2026-07-28T15-51-32.196Z-109284-ba9d8612-3c57-40dc-b007-b126a3651528/manifest.json], symbol: [entity-4.24-bumper-holdable-dash-smuggle], note: [2026-07-28 在受锁主工作区的物理 `vendor/celeste-game` 上运行；isolated save/tmp、动态 loopback ports、nonce 与 spawned Celeste PID 精确匹配，并完成受控清理。121 帧完整逐帧比较 position、speed、state、facing、dashes、stamina、grounded、ducking 与 death：position 最大误差为 0，speed 最大误差为 `0.000008`，其余七字段均一致。]),
  candidate-e2e: none,
)
