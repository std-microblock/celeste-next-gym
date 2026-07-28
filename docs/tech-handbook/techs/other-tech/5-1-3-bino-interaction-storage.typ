#import "../../template.typ": tech, evidence

#tech(
  id: "5.1.3",
  title-zh: "望远镜交互存储",
  title-en: "Bino Interaction Storage",
  status: "unimplemented",
  description-zh: [望远镜交互标志可在玩家状态已恢复 Normal 后继续保留，并通过房间切换在新的位置重新触发交互流程。],
  description-en: [Room removal can restore the player to Normal without clearing Lookout.interacting; that entity-owned flag and TalkComponent lifecycle are the basis of interaction storage. Rust preserves the mismatch; real cross-room proof remains pending.],
  source-evidence: evidence(
    path: [Celeste/Lookout.cs; Celeste/Level.cs],
    symbol: [Lookout.Removed; Lookout.StopInteracting; Level.TransitionRoutine],
    snippet: raw(block: true, lang: "cs", "public override void Removed(Scene scene) {\n    if (interacting) {\n        Player player = scene.Tracker.GetEntity<Player>();\n        if (player != null) player.StateMachine.State = StNormal;\n    }\n}\npublic void StopInteracting() {\n    interacting = false;\n}"),
    note: [`Removed` 在房间卸载时只恢复 Player 状态，并没有调用 `StopInteracting` 或清除实体标志；Player Normal 与 Lookout 交互生命周期因此可以脱钩，形成跨房间储存所需的不一致状态。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [booster_boosting; update_transition], note: [Booster 的 BoostingPlayer 生命周期在 Dash 结束前禁止同一 Booster 重入；切换完成时，旧房 Lookout 标记 removed 并恢复 Player Normal，但故意保留 `interacting=true`。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [bino_interaction_storage_uses_a_native_booster_after_dummy_walk], note: [本地回归以真实 Booster 打断 DummyWalk，并验证 Dash 生命周期、过渡贴地碰撞查询和第二房间切换；跨房间交互标志仍由 `bino_interaction_storage_survives_lookout_room_removal` 覆盖。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.1.3-bino-interaction-storage.ts; scripts/e2e-real/scenarios/lookout-parts.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs], symbol: [other-5.1.3-bino-interaction-storage; TECH_OTHER_5_1_3_BINO_INTERACTION_STORAGE; IsLookoutInteracting], note: [候选 fixture 已改为有效的独立双房地图：Lookout `x=938`，Booster `[924,491,16,16]`，f120 后向右走以触发房间切换；collector 在 `Lookout.Removed` 原处理前捕获 `interacting`。尚未对该 rebased 版本运行真实 Everest，因此保持 candidate。]),
)
