#import "../../template.typ": tech, evidence

#tech(
  id: "4.10.4",
  title-zh: "无抓墙携物 Dream Hyper",
  title-en: "Holdable Grabless Dream Hyper",
  status: "unimplemented",
  description-zh: [出口侧无法抓墙时，可先松开投掷物，利用 Dream Jump／土狼窗口做 Hyper，再重新抓回物品。],
  description-en: [When the exit wall cannot be grabbed, release the holdable, use dream-exit jump timing to hyper, then catch the item again.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Source/Holdable.cs],
    symbol: [Player.DreamDashEnd / Player.Throw / Holdable.Release],
    snippet: raw(block: true, lang: "cs", "if (DashDir.X != 0f)\n    jumpGraceTimer = 0.1f;\n...\nHolding.Release(force);\nHolding = null;"),
    note: [无抓墙变体直接使用水平 DreamDashEnd 的 0.1 秒 grace；必须在出口后松物、越过 CannotHold 窗口、完成 Hyper 并重新抓物，不能用普通出口或单纯 regrab 代替。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [interact / release_theo / super_jump / try_pickup_theo]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [holdable_grabless_dream_hyper_uses_exit_grace_without_a_climb_state]),
  e2e-evidence: none,
  candidate-e2e: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [entity-4.10.4-holdable-grabless-dream-hyper],
    note: [真实候选已观测 `exit=49, release=53, blocked=true, regrab=true`，但 `hyper=-1`；出口、投掷、CannotHold 与回抓证据不足以证明核心 Hyper，因此保持未实现。],
  ),
)
