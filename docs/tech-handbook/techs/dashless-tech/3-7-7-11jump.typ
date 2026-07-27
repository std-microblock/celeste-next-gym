#import "../../template.typ": tech, evidence

#tech(
  id: "3.7.7",
  title-zh: "11 格跳",
  title-en: "11jump",
  status: "implemented",
  description-zh: [房间切换 Cornerboost 配合切换期间的多次缓冲攀跳，可在极端子像素条件下跨越 11 格。],
  description-en: [A transition cornerboost plus multiple buffered climb jumps during the room change can span eleven tiles.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.TransitionTo; Player.OnTransition; Player.ClimbJump],
    snippet: raw(block: true, lang: "cs", "if (Position == target) {\n    ZeroRemainderX();\n    ZeroRemainderY();\n    Speed.X = (int)Math.Round(Speed.X);\n    Speed.Y = (int)Math.Round(Speed.Y);\n    return true;\n}\n...\nRefillDash();\nRefillStamina();\n...\nif (!onGround)\n    Stamina -= ClimbJumpCost;"),
    note: [TransitionTo 以每秒 60 像素移向目标，并在抵达当帧清除 movement remainder、取整速度；资源直到随后 OnTransition 才恢复。切换后的缓冲攀跳各扣 27.5 体力，碰角时保存的水平速度让三连攀跳跨过 11 格缺口。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [begin_transition; update_transition; climb_bounds_check; climb_jump], note: [切房使用当前房间边界完成墙面探测，并在抵达目标的同一帧取整速度、下一阶段恢复资源。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [eleven_jump_buffers_three_climb_jumps_across_a_room_transition]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/eleven-jump.ts], symbol: [eleven-jump; verifyElevenJump], note: [独立双房间 MapPart 记录 121 个真实状态；state 41 完成切换并恢复资源，states 42–44 体力依次为 82.5、55、27.5，随后以 191.66666 retained speed 跨过 11 格并落地。九类核心字段逐帧一致，最大位置误差 0、速度误差 0.000001。]),
  candidate-e2e: none,
)
