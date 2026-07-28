#import "../../template.typ": tech, evidence

#tech(
  id: "4.18.3",
  title-zh: "Core 方块实体位移",
  title-en: "Core Block Entity Displacement (ced)",
  status: "unimplemented",
  description-zh: [在 Core 方块恢复附属实体前再次打碎或移动它，可让尖刺等 static mover 在错误位置重生。],
  description-en: [BounceBlock reform moves disabled StaticMovers to the restored body before a 0.35-second alarm re-enables them. Rust models that ordering, but the real candidate kept the player inside the source body so BlockedCheck prevented reform through frame 280; entity displacement therefore remains unimplemented.],
  source-evidence: evidence(
    path: [Celeste/BounceBlock.cs],
    symbol: [BounceBlock.Update],
    snippet: raw(block: true, lang: "cs", "MoveStaticMovers(Position - oldPosition);\nCollidable = true;\nstate = States.Waiting;\nAlarm.Set(this, 0.35f, EnableStaticMovers);"),
    note: [方块先移动仍 disabled 的附属物并恢复 body，0.35 秒 Alarm 之后才重新启用 StaticMover；若方块在 Alarm 窗口再次移动，就可能把附属尖刺留在偏移位置。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [BounceBlockSnapshot.static_movers_enabled / advance_bounce_blocks]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [core_block_moves_disabled_spikes_before_the_reform_alarm_reenables_them / core_block_candidate_clears_source_body_before_reform_blocked_check]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.18.3-core-block-entity-displacement.ts], symbol: [entity-4.18.3-core-block-entity-displacement], note: [最终真实候选能观测破碎 body 与 disabled StaticMover，但 280 帧内没有出现 body 已恢复而 spike 仍 disabled 的中间态，语义门报 `block body did not reform`；保持未实现。]),
)
