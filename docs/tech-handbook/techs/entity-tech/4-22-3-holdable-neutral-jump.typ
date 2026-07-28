#import "../../template.typ": tech, evidence

#tech(
  id: "4.22.3",
  title-zh: "携物 Neutral",
  title-en: "Holdable Neutral Jump",
  status: "unimplemented",
  description-zh: [Theo 可直接配合 Neutral；水母通常需要先中性放下、执行 Neutral 墙跳，再重新抓取。],
  description-en: [Theo can accompany normal neutral jumps, while jellies usually require a neutral drop, neutral wall jump, and regrab.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.NormalUpdate / Player.ClimbJump / Player.WallJump],
    snippet: raw(block: true, lang: "cs", "if (Facing == Facings.Right && Input.Grab.Check && Stamina > 0 && Holding == null)\n    ClimbJump();\nelse\n    WallJump(-1);"),
    note: [Theo 仍在 Holding 时，即使按住 Grab 也不满足 ClimbJump 的 Holding == null 条件，于是同一次墙跳走普通 WallJump，保留 Theo 并得到 Neutral 的 -130／-105 起速。Rust 先前漏掉该条件，本批已修复；Jelly 的放下重抓分支仍缺 Glider runtime。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [normal_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [held_theo_turns_grabbed_wall_jump_into_a_normal_neutral]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.22.3-holdable-neutral-jump.ts], symbol: [entity-4.22.3-holdable-neutral-jump], note: [独立墙面 MapPart 要求 Theo 全程保持 Holding，并观测普通 WallJump 的 -130／-105；真实 Everest 尚待 FIFO 锁内采集。]),
)
