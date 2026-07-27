#import "../../template.typ": tech, evidence

#tech(
  id: "5.6",
  title-zh: "Kermit Dash",
  title-en: "Kermit Dash",
  status: "implemented",
  description-zh: [向上冲刺越过房间边界时 BeforeUpTransition 会取消 Dash 状态，却不清除 dashAttackTimer 或 DashDir；切房期间计时暂停，因此新房间仍可触发定向冲刺交互。],
  description-en: [Crossing an upward room boundary cancels Dash state without clearing dashAttackTimer or DashDir; transition frames pause the timer, preserving directional dash interactions in the next room.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste/Level.cs],
    symbol: [Player.BeforeUpTransition; Player.TransitionTo; Player.DashAttacking; Player.OnCollideV; Level.TransitionRoutine],
    snippet: raw(block: true, lang: "cs", "public void BeforeUpTransition() {\n    Speed.X = 0;\n    varJumpSpeed = Speed.Y = JumpSpeed;\n    StateMachine.State = StNormal;\n    AutoJump = true;\n}\npublic bool DashAttacking => dashAttackTimer > 0 || StateMachine.State == StRedDash;\n...\nif (DashAttacking && data.Direction.Y == Math.Sign(DashDir.Y))\n    data.Hit.OnDashCollide(this, data.Direction);"),
    note: [BeforeUpTransition 只改速度、状态与跳跃字段，没有清除 0.3 秒 dashAttackTimer 或 DashDir；TransitionRoutine 期间普通 Player.Update 不运行，所以进入新房后 DashAttacking 仍携带原方向。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [PlayerSnapshot.dash_dir; PlayerSnapshot.dash_attack_timer; begin_transition; update_transition; interact], note: [transition_timer 分支先于 tick_timers 返回，精确保留攻击计时；begin_transition 将普通向上 Dash 改为 Normal，但不改 dash_dir。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [kermit_dash_preserves_attack_and_direction_through_vertical_transition], note: [回归测试断言切房首帧已为 Normal、方向仍为 0/-1、攻击计时冻结，完成后以该窗口打破新房间的护盾实体并进入 StarFly。]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.6-kermit-dash.ts], symbol: [other-5.6-kermit-dash; verifyKermitDash], note: [独立双房间 MapPart 从真实上冲开始，语义守卫在 Dash→Normal 首帧和资源恢复完成帧读取 Everest DashDir/dashAttackTimer，确认完整切房后方向与攻击窗口仍保留；九类核心字段逐帧容差不超过 0.01。]),
  candidate-e2e: none,
)
