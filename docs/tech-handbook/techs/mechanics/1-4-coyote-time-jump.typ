#import "../../template.typ": tech, evidence

#tech(
  id: "1.4",
  title-zh: "土狼时间／土狼跳",
  title-en: "Coyote Time/Jump",
  status: "implemented",
  description-zh: [离开地面后的 5 帧内仍可按地面条件起跳，也能在这段窗口内发动 Super 或 Hyper。],
  description-en: [For five frames after leaving a ledge, the player may still jump or start grounded dash tech such as supers and hypers.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.Update; Player.NormalUpdate],
    snippet: raw(block: true, lang: "cs", "if (onGround)\n    jumpGraceTimer = JumpGraceTime;\nelse if (jumpGraceTimer > 0)\n    jumpGraceTimer -= Engine.DeltaTime;\n...\nif (Input.Jump.Pressed && jumpGraceTimer > 0)\n    Jump();"),
    note: [接地时设置 0.1 s jumpGraceTimer，离地后逐帧扣减并在 NormalUpdate 中优先消费。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [step; normal_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [coyote_jump_consumes_source_grace_window_after_leaving_a_ledge]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [coyote-jump], note: [离开平台后第 3 输入帧起跳；九类核心字段逐帧比较，最大误差 0。]),
  candidate-e2e: none,
)
