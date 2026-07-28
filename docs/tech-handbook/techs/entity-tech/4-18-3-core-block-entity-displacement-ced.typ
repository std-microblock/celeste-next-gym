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
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.18.3-core-block-entity-displacement.ts; scripts/e2e-real/scenarios/reform-parts.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs], symbol: [entity-4.18.3-core-block-entity-displacement; TECH_ENTITY_4_18_3_CORE_BLOCK_ENTITY_DISPLACEMENT; SnapshotCapture.Capture], note: [2026-07-28 隔离真实 Everest run `2026-07-28T17-36-38.746Z-100144-9db259b6-006d-47f8-94c5-9c22eb6ab9c1` 完成物理 vendor 安装校验、nonce/精确 child PID 认证及受控清理。方块在 frame 35 破碎后停于 `(686,425)`，附属 spike 同为 `(750,425)`；玩家从 frame 81 起位于 `(716,496)`，但至 frame 220 仍未出现 `reformBlockCollidable=true`、`reformSpikeCollidable=false`。semantic gate 报 `block body did not reform while its StaticMover remained disabled`，故未取得九字段差分通过，保持未实现。]),
)
