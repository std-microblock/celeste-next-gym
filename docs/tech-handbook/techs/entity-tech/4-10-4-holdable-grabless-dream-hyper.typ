#import "../../template.typ": tech, evidence

#tech(
  id: "4.10.4",
  title-zh: "无抓墙携物 Dream Hyper",
  title-en: "Holdable Grabless Dream Hyper",
  status: "implemented",
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
  e2e-evidence: evidence(
    path: [scripts/e2e-real/scenarios/playground/entity-4.10.4-holdable-grabless-dream-hyper.ts],
    symbol: [entity-4.10.4-holdable-grabless-dream-hyper],
    note: [独立 DreamBlock／Theo MapPart 的真实轨迹依次观测水平 Dream 出口、松物、0.1 秒 CannotHold 阻止立即回抓，以及出口 grace 上的 325 Hyper。101 个状态的 position／speed 最大误差均为 0，其余七类字段逐帧一致。],
  ),
  candidate-e2e: none,
)
