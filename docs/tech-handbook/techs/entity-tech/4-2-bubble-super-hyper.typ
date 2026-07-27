#import "../../template.typ": tech, evidence

#tech(
  id: "4.2",
  title-zh: "泡泡 Super／Hyper",
  title-en: "Bubble Super / Hyper",
  status: "implemented",
  description-zh: [在离地后的土狼窗口进入泡泡并输入 Super 或 Hyper，可带着对应速度离开泡泡，同时保留泡泡提供的冲刺。],
  description-en: [Entering a bubble during coyote time and performing a super or hyper carries the boost out while retaining the bubble-granted dash.],
  source-evidence: evidence(path: [Source/Player/Player.cs], symbol: [Player.NormalUpdate / Player.BoostUpdate / Player.SuperJump], note: [Boost 状态补充冲刺但不会清除进入前的 jumpGraceTimer；泡泡内启动水平 Dash 后，土狼窗口中的跳跃沿用 SuperJump，并保留泡泡补充的冲刺。]),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [normal_update / boost_update / dash_update / super_jump]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [bubble_super_uses_coyote_grace_and_keeps_the_refilled_dash / bubble_demohyper_uses_coyote_grace_and_keeps_the_refilled_dash]),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [entity-4.2-bubble-super / entity-4.2-bubble-demohyper],
    note: [两个独立 Playground 场景均在离开 jumpthrough 后的 0.1 秒 grace 内进入标准绿色 Booster，并在泡泡内手动水平 dash 后跳出。每项各 11 个状态，position/speed 最大误差均为 0，其余七类字段逐帧一致；Super 终态为 `(260,-105)`，Demo Hyper 终态为 `(325,-52.5)`，且两者 dashes 均为 1，确认泡泡补充的冲刺未被消耗。],
  ),
  candidate-e2e: none,
)
