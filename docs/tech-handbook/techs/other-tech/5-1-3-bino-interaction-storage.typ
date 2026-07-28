#import "../../template.typ": tech, evidence

#tech(
  id: "5.1.3",
  title-zh: "望远镜交互存储",
  title-en: "Bino Interaction Storage",
  status: "implemented",
  description-zh: [望远镜交互标志可在玩家状态已恢复 Normal 后继续保留，并通过房间切换在新的位置重新触发交互流程。],
  description-en: [Room removal restores the player to Normal without clearing Lookout.interacting; the entity-owned flag persists across the completed side transition, as verified against a real Everest run.],
  source-evidence: evidence(
    path: [Celeste/Lookout.cs; Celeste/Level.cs],
    symbol: [Lookout.Removed; Lookout.StopInteracting; Level.TransitionRoutine],
    snippet: raw(block: true, lang: "cs", "public override void Removed(Scene scene) {\n    if (interacting) {\n        Player player = scene.Tracker.GetEntity<Player>();\n        if (player != null) player.StateMachine.State = StNormal;\n    }\n}\npublic void StopInteracting() {\n    interacting = false;\n}"),
    note: [`Removed` 在房间卸载时只恢复 Player 状态，并没有调用 `StopInteracting` 或清除实体标志；Player Normal 与 Lookout 交互生命周期因此可以脱钩，形成跨房间储存所需的不一致状态。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [LookoutSnapshot.removed; update_transition], note: [切换完成时，旧房 Lookout 标记 removed 并恢复 Player Normal，但故意保留 `interacting=true`；被移除实体不再持有相机。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [bino_interaction_storage_survives_lookout_room_removal], note: [双房回归验证 transition 完成后 `Normal + removed + interacting` 三者同时成立。]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.1.3-bino-interaction-storage.ts; .tmp/e2e-runs/2026-07-28T18-49-11.746Z-93192-db0c99a6-da91-48d8-a1a8-509b73120026/manifest.json; .tmp/e2e-other-5.1.3-bino-interaction-storage-trace.json; crates/celeste-physics/examples/compare_real_trace.rs], symbol: [other-5.1.3-bino-interaction-storage; TECH_OTHER_5_1_3_BINO_INTERACTION_STORAGE; lookoutRemovalObserved; lookoutRemovedWhileInteracting; compare_real_trace], note: [2026-07-29 在物理 `vendor/celeste-game` 上以隔离 save/tmp、动态端口、nonce 和 spawned child PID 116668 握手运行 301 帧；position 与 speed 最大误差均为 0，state、facing、dashes、stamina、grounded、ducking 与 death 逐帧一致。语义门确认原生 Booster 打断 Dummy，Lookout.Removed 时仍保留 interacting，并完成侧向转场。]),
)
