#import "../../template.typ": tech, evidence

#tech(
  id: "2.5",
  title-zh: "延长冲刺技巧",
  title-en: "Extended Dashes",
  status: "implemented",
  description-zh: [冲刺恢复冷却短于完整冲刺时长，因此在冷却结束后、冲刺状态结束前跳出 Super／Hyper／Wavedash，可以重新获得冲刺。],
  description-en: [Because dash refill cooldown ends before the dash state, jumping during the late window of a super, hyper, or wavedash can restore the spent dash.],
  source-evidence: evidence(path: [Source/Player/Player.cs], symbol: [Player.Update; Player.DashUpdate], note: [0.1 s dashRefillCooldown 先于冲刺状态结束，晚跳时地面恢复 Dash 后再退出冲刺。]),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [step; tick_timers; dash_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [extended_super_refills_dash_before_late_dash_jump]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [extended-super], note: [独立场景第 11 状态以 260/-105 跳出且 dashes=1；max speed error 0。]),
  candidate-e2e: none,
)
