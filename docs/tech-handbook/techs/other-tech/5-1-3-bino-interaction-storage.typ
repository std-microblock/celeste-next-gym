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
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [LookoutSnapshot.removed; update_transition], note: [切换完成时，旧房 Lookout 标记 removed 并恢复 Player Normal，但故意保留 `interacting=true`；被移除实体不再持有相机。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [bino_interaction_storage_survives_lookout_room_removal], note: [双房回归验证 transition 完成后 `Normal + removed + interacting` 三者同时成立。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.1.3-bino-interaction-storage.ts; scripts/e2e-real/scenarios/lookout-parts.ts; mods/CelesteGymCollector/Source/CelesteGymCollectorModule.cs; crates/celeste-physics/src/sim.rs; Celeste/Player/Player.cs; Celeste/Booster.cs; Celeste/Lookout.cs], symbol: [other-5.1.3-bino-interaction-storage; TECH_OTHER_5_1_3_BINO_INTERACTION_STORAGE; LookoutRemoved; bino_interaction_storage_uses_a_native_booster_after_dummy_walk; Player.Update; Booster.Booster; Player.BoostUpdate; Lookout.LookRoutine; Player.DummyWalkToExact], note: [2026-07-29 前两次独立 Everest 运行均未进入原生 Booster：首次确认 Lookout Position 为 x=940、DummyWalk 停在 Player X=932，center X=952 的 Booster 未触发；第二次 trace 显示继承的 playground.base 右墙 x=936 将 Player 固定在 x=932。专用无右墙双房 fixture 的第三次重跑在 frame 8 逐帧门失败：Rust 已在 x=920 进入 Boost，Everest 仍为 Dummy（speed 64）。源码确认 `Booster` 使用 `Circle(10, 0, 2)` 的 `PlayerCollider`，而 `Player.Update` 在遍历所有 PlayerCollider 前临时换入 8x9 hurtbox；Rust 先前误用 8x11 movement collider，使边缘圆形相交早一帧。第四次运行（bef2691）已将首次 Booster overlap 对齐到 frame 9，但完整比较仍在 frame 10 失败。后续源码修正使 phase-1 Lookout 仅在协程启动时写入 Dummy、保留 PlayerCollider 所设 Boost；并使 BoostUpdate 不再重置由后置 DummyWalk 写入的速度。最后确认 DummyWalk 的源目标为 Lookout X=940（不是 X-8），并只在协程开始时确定 Facing，使本地 frame 9/10、16、18 的核心字段对齐；旧 trace 的首个残余差异后移到 frame 33 state，故仍保持 candidate，暂不重跑。]),
)
