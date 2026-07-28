#import "../../template.typ": tech, evidence

#tech(
  id: "4.18.3",
  title-zh: "Core 方块实体位移",
  title-en: "Core Block Entity Displacement (ced)",
  status: "implemented",
  description-zh: [在 Core 方块恢复附属实体前再次打碎或移动它，可让尖刺等 static mover 在错误位置重生。],
  description-en: [BounceBlock reform moves disabled StaticMovers to the restored body before a 0.35-second alarm re-enables them. The verified fixture waits on a one-pixel-separated right platform until the body has passed its real BlockedCheck, then crosses during the disabled-StaticMover alarm.],
  source-evidence: evidence(
    path: [Celeste/BounceBlock.cs],
    symbol: [BounceBlock.Update],
    snippet: raw(block: true, lang: "cs", "MoveStaticMovers(Position - oldPosition);\nCollidable = true;\nstate = States.Waiting;\nAlarm.Set(this, 0.35f, EnableStaticMovers);"),
    note: [方块先移动仍 disabled 的附属物并恢复 body，0.35 秒 Alarm 之后才重新启用 StaticMover；若方块在 Alarm 窗口再次移动，就可能把附属尖刺留在偏移位置。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [BounceBlockSnapshot.static_movers_enabled / advance_bounce_blocks]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [core_block_moves_disabled_spikes_before_the_reform_alarm_reenables_them / core_block_candidate_clears_source_body_before_reform_blocked_check]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.18.3-core-block-entity-displacement.ts; scripts/e2e-real/scenarios/reform-parts.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs; .tmp/e2e-runs/2026-07-28T22-11-23.942Z-98824-a6af9743-c496-42a4-8213-67a3f9613666/manifest.json], symbol: [entity-4.18.3-core-block-entity-displacement; TECH_ENTITY_4_18_3_CORE_BLOCK_ENTITY_DISPLACEMENT; SnapshotCapture.Capture], note: [2026-07-28 的物理 `vendor/celeste-game` 隔离 Everest run 使用候选 SHA `2cc21f05898bce7efe5f8977c6d5ca042baa3c8f`；动态端口 `58623/58624`、nonce `1a073b0b-7182-49d7-9cac-78c6548af7e4` 与 spawned Celeste PID `90656` 精确认证，per-run save/tmp 隔离，cleanup 仅终止该受控子进程且已完成。221 个状态逐帧比较 position、speed、state、facing、dashes、stamina、grounded、ducking、death，最大 position 误差为 `0`、最大 speed 误差为 `0.000047`。真实 trace 在 f35 body/spike 为 `(691,468)`/`(755,468)` 且均 disabled；f132 body 在 `(712,480)` 重组、spike 仍 disabled；f153 spike 在由真实 broken→restored 位移导出的 `(776,480)` 重新启用，alarm 间隔 21 帧。]),
  candidate-e2e: none,
)
