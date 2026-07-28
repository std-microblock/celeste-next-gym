#import "../../template.typ": tech, evidence

#tech(
  id: "4.18.3",
  title-zh: "Core 方块实体位移",
  title-en: "Core Block Entity Displacement (ced)",
  status: "unimplemented",
  description-zh: [在 Core 方块恢复附属实体前再次打碎或移动它，可让尖刺等 static mover 在错误位置重生。],
  description-en: [BounceBlock reform moves disabled StaticMovers to the restored body before a 0.35-second alarm re-enables them. The replacement candidate first drives the player onto a separated landing, lets the body pass its real BlockedCheck, then returns during the disabled-StaticMover alarm; it remains unimplemented until that physical Everest path is verified.],
  source-evidence: evidence(
    path: [Celeste/BounceBlock.cs],
    symbol: [BounceBlock.Update],
    snippet: raw(block: true, lang: "cs", "MoveStaticMovers(Position - oldPosition);\nCollidable = true;\nstate = States.Waiting;\nAlarm.Set(this, 0.35f, EnableStaticMovers);"),
    note: [方块先移动仍 disabled 的附属物并恢复 body，0.35 秒 Alarm 之后才重新启用 StaticMover；若方块在 Alarm 窗口再次移动，就可能把附属尖刺留在偏移位置。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [BounceBlockSnapshot.static_movers_enabled / advance_bounce_blocks]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [core_block_moves_disabled_spikes_before_the_reform_alarm_reenables_them / core_block_candidate_clears_source_body_before_reform_blocked_check]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.18.3-core-block-entity-displacement.ts; scripts/e2e-real/scenarios/reform-parts.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs; Celeste/CassetteBlock.cs], symbol: [entity-4.18.3-core-block-entity-displacement; TECH_ENTITY_4_18_3_CORE_BLOCK_ENTITY_DISPLACEMENT; SnapshotCapture.Capture; CassetteBlock.Update], note: [2026-07-28 隔离真实 Everest run `2026-07-28T17-36-38.746Z-100144-9db259b6-006d-47f8-94c5-9c22eb6ab9c1` 在 frame 35 破碎后至 frame 220 仍未观测 body reform；f131 才开始离开原位置，因而不能证明 BounceBlock 的 BlockedCheck 已获准。重建候选在第 36–107 帧通过两级右侧 jump-thru 离开 source，第 118 帧预缓冲上左 dash；场景在 body reformed / spike disabled 帧显式拒绝 source-player overlap，再要求 0.35 秒后 spike re-enable 且已随二次 bounce 偏移。CassetteBlock 的 `BlockedCheck`/最多 4px actor wiggle 是同类实体更新先判阻塞、后启用碰撞的对照：不能用人工状态开关替代真实离开 source 的输入路径。尚未运行新真实 E2E，保持未实现。]),
)
