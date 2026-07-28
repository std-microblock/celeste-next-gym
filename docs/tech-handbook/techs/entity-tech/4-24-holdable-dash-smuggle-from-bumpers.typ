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
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [initialize_bumpers / advance_bumpers / interact / launch_update / try_pickup_theo], note: [Bumper advances its randomized SineWave and publishes `Position` before its PlayerCollider invokes `OnPlayer`. The physical trace records f85 `(132.59096,492.19797)` and f86 `(132.72580,492.24374)`; f86's non-horizontal launch is therefore computed from the newly published f86 target, then receives the matching-direction 1.2x horizontal boost. `Player.LaunchUpdate` then scans Holdables without a `Holding == null` guard: when Grab survives the freeze, the held Theo overlaps its own pickup collider and restarts PickupCoroutine at f93. The regression locks f91–f95's Launch-to-Pickup boundary without changing collision radius or tolerance.]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [bumper_freeze_smuggle_releases_dashes_and_regrabs_theo / bumper_smuggle_releases_down_after_buffered_diagonal_dash_to_regrab_theo]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.24-bumper-holdable-dash-smuggle.ts], symbol: [entity-4.24-bumper-holdable-dash-smuggle], note: [物理 vendor 运行 `2026-07-28T15-38-58.760Z-109588-861e4e95-df62-48ef-8a38-39b8376c5fab` 使用隔离 save/tmp、动态 loopback ports、nonce `fc0ccfa6-2754-432a-ba24-8006e3dd91a1` 与 spawned Celeste PID `110620` 握手，随后受控清理。该真实 trace 的当前 Rust 重放已逐帧比较 121 个状态：position 最大误差 0、speed 最大误差 `0.000008`，其余七字段一致；f93 按源 LaunchUpdate 的无 Holding guard 进入 Pickup。仍保持 candidate，待当前提交通过新的受控物理 E2E 后转正。]),
)
