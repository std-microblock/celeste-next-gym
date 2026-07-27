#import "../../template.typ": tech, evidence

#tech(
  id: "1.5",
  title-zh: "冲刺攻击窗口",
  title-en: "Dash Attack",
  status: "implemented",
  description-zh: [冲刺结束后仍保留约 6 帧 Dash Attack，可触发墙反、冲刺开关等交互；跳跃、攀跳、墙跳或抓取会提前取消它。],
  description-en: [A roughly six-frame dash-attack window survives after dash movement ends, enabling dash interactions until a jump, wall action, or grab cancels it.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.DashBegin; Player.DashCoroutine; Player.DashAttacking; Player.OnCollideH],
    snippet: raw(block: true, lang: "cs", "private const float DashTime = .15f;\nprivate const float DashAttackTime = .3f;\n...\ndashAttackTimer = DashAttackTime;\n...\nyield return DashTime;\n...\npublic bool DashAttacking => dashAttackTimer > 0 || StateMachine.State == StRedDash;"),
    note: [DashBegin 同时打开 0.3 秒攻击计时，而 DashCoroutine 的位移状态只持续 0.15 秒，因此普通冲刺结束后仍保留约 0.15 秒实体交互窗口；跳跃和墙动作会把计时清零。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [PlayerSnapshot.dash_attack_timer; tick_timers; begin_dash; dash_update; interact]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [dash_attack_survives_dash_end_and_breaks_a_late_feather_shield]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [mechanics-dash-attack-late-shield], note: [真实第 15 帧已回到 Normal 且 dashAttackTimer=0.11666633；第 19 帧仍以 0.049999546 窗口触发护盾并进入 StarFly。41 个状态帧九类字段逐帧一致，max position error 0，max speed error 0.000011。]),
  candidate-e2e: none,
)
