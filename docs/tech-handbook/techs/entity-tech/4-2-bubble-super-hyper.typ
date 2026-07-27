#import "../../template.typ": tech, evidence

#tech(
  id: "4.2",
  title-zh: "泡泡 Super／Hyper",
  title-en: "Bubble Super / Hyper",
  status: "implemented",
  description-zh: [在离地后的土狼窗口进入泡泡并输入 Super 或 Hyper，可带着对应速度离开泡泡，同时保留泡泡提供的冲刺。],
  description-en: [Entering a bubble during coyote time and performing a super or hyper carries the boost out while retaining the bubble-granted dash.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.Update / Player.BoostBegin / Player.BoostUpdate / Player.DashUpdate / Player.SuperJump],
    snippet: raw(block: true, lang: "cs", "private const float DuckSuperJumpXMult = 1.25f;\nprivate const float DuckSuperJumpYMult = .5f;\nprivate const float JumpGraceTime = 0.1f;\nprivate const float JumpSpeed = -105f;\nprivate const float SuperJumpH = 260f;\n\nif (onGround) jumpGraceTimer = JumpGraceTime;\nelse if (jumpGraceTimer > 0) jumpGraceTimer -= Engine.DeltaTime;\n\nprivate void BoostBegin() { RefillDash(); RefillStamina(); }\nif (Input.Dash.Pressed) return boostRed ? StRedDash : StDash;\n\nif (DashDir.Y == 0 && CanUnDuck && Input.Jump.Pressed && jumpGraceTimer > 0) {\n    SuperJump();\n    return StNormal;\n}\n\nSpeed.X = SuperJumpH * (int)Facing;\nSpeed.Y = JumpSpeed;\nif (Ducking) {\n    Speed.X *= DuckSuperJumpXMult;\n    Speed.Y *= DuckSuperJumpYMult;\n}"),
    note: [Player.Update 先在着地时写入 0.1 秒 jump grace，离地后逐帧递减；进入 Boost 只补充 dash/stamina，不清除该 timer。BoostUpdate 检测手动冲刺后直接切到 Dash，随后的水平 DashUpdate 在 jump buffer 与正 grace 同时成立时调用 SuperJump。普通 Super 写入 `(260,-105)`；蹲伏的 demo dash 复用同一路径并乘 `(1.25,0.5)`，得到 `(325,-52.5)`，而泡泡补充的 dash 没有被 StartDash 消耗。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [normal_update / boost_update / dash_update / super_jump]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [bubble_super_uses_coyote_grace_and_keeps_the_refilled_dash / bubble_demohyper_uses_coyote_grace_and_keeps_the_refilled_dash]),
  e2e-evidence: evidence(
    path: [scripts/e2e-real/scenarios/playground],
    symbol: [entity-4.2-bubble-super / entity-4.2-bubble-demohyper],
    note: [两个独立 Playground 场景均在离开 jumpthrough 后的 0.1 秒 grace 内进入标准绿色 Booster，并在泡泡内手动水平 dash 后跳出。每项各 11 个状态，position/speed 最大误差均为 0，其余七类字段逐帧一致；Super 终态为 `(260,-105)`，Demo Hyper 终态为 `(325,-52.5)`，且两者 dashes 均为 1，确认泡泡补充的冲刺未被消耗。],
  ),
  candidate-e2e: none,
)
