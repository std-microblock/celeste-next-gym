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
    note: [Theo 仍在 Holding 时，即使按住 Grab 也不满足 ClimbJump 的 Holding == null 条件，于是走普通 WallJump 并得到 -130／-105。Jelly 分支先放下、在 0.3 秒锁定中执行普通墙跳，再于锁定结束后重抓；两条 Rust 路径现均有回归。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [normal_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [held_theo_turns_grabbed_wall_jump_into_a_normal_neutral / jelly_neutral_drop_wall_jump_regrabs_after_long_lockout]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.22.3-holdable-neutral-jump.ts / scripts/e2e-real/scenarios/playground/entity-4.22.3-jelly-neutral-jump.ts], symbol: [entity-4.22.3-holdable-neutral-jump / entity-4.22.3-jelly-neutral-jump], note: [Theo 与 Jelly 各有独立墙面 MapPart；真实 Jelly 首轮在第 24 帧完成放下，但没有形成普通 Neutral 墙跳（neutral = -1），因此双变体证据尚未闭环并保留 candidate。]),
)
