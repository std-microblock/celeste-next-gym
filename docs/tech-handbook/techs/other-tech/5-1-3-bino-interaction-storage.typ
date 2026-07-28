#import "../../template.typ": tech, evidence

#tech(
  id: "5.1.3",
  title-zh: "望远镜交互存储",
  title-en: "Bino Interaction Storage",
  status: "unimplemented",
  description-zh: [望远镜交互标志可在玩家状态已恢复 Normal 后继续保留，并通过房间切换在新的位置重新触发交互流程。],
  description-en: [Room removal can restore the player to Normal without clearing Lookout.interacting; that entity-owned flag and TalkComponent lifecycle are the basis of interaction storage. Rust does not model either, so the technique is not implemented.],
  source-evidence: evidence(
    path: [Celeste/Lookout.cs; Celeste/Level.cs],
    symbol: [Lookout.Removed; Lookout.StopInteracting; Level.TransitionRoutine],
    snippet: raw(block: true, lang: "cs", "public override void Removed(Scene scene) {\n    if (interacting) {\n        Player player = scene.Tracker.GetEntity<Player>();\n        if (player != null) player.StateMachine.State = StNormal;\n    }\n}\npublic void StopInteracting() {\n    interacting = false;\n}"),
    note: [`Removed` 在房间卸载时只恢复 Player 状态，并没有调用 `StopInteracting` 或清除实体标志；Player Normal 与 Lookout 交互生命周期因此可以脱钩，形成跨房间储存所需的不一致状态。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/sim.rs], symbol: [Map; update_transition], note: [Rust 房间切换只更新 bounds 与 Player 资源，没有逐房实体卸载、Lookout/TalkComponent 或可携带的 interaction flag；无法表示源码中的脱钩状态。]),
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
