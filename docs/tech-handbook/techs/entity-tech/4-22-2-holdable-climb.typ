#import "../../template.typ": tech, evidence

#tech(
  id: "4.22.2",
  title-zh: "携物攀墙",
  title-en: "Holdable Climb",
  status: "implemented",
  description-zh: [中性放下物品后立刻攀跳，再快速重抓，可在保留物品的同时完成攀墙动作。],
  description-en: [Neutral-drop the item, climb jump immediately, and regrab it to climb while keeping the holdable.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.Drop / Player.NormalUpdate / Player.ClimbUpdate / Player.ClimbJump],
    snippet: raw(block: true, lang: "cs", "Holding.Release(Vector2.Zero);\nHolding = null;\n...\nif (Holding == null && Input.Grab.Check) return StClimb;\n...\nif (Input.Jump.Pressed) ClimbJump();"),
    note: [携物时 NormalUpdate 不允许进入 Climb；先中性放下会清空 Holding，下一帧可抓墙，再由 ClimbUpdate 执行 ClimbJump。CannotHold 结束后回到 Normal 才能重抓物品。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [release_theo / normal_update / climb_update / try_pickup_theo]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [neutral_drop_climb_jump_regrabs_theo_after_the_lockout]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.22.2-holdable-climb.ts], symbol: [entity-4.22.2-holdable-climb], note: [独立 Theo + 墙面 MapPart 的真实 Everest 轨迹完整通过 pickup→neutral drop→Climb→ClimbJump→regrab。51 个状态的九类核心字段逐帧一致，最大 position／speed 误差均为 0。]),
  candidate-e2e: none,
)
