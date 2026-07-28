#import "../../template.typ": tech, evidence

#tech(
  id: "4.18.3",
  title-zh: "Core 方块实体位移",
  title-en: "Core Block Entity Displacement (ced)",
  status: "unimplemented",
  description-zh: [在 Core 方块恢复附属实体前再次打碎或移动它，可让尖刺等 static mover 在错误位置重生。],
  description-en: [BounceBlock reform moves disabled StaticMovers to the restored body before a 0.35-second alarm re-enables them. The replacement candidate waits on a one-pixel-separated right platform until the body has passed its real BlockedCheck, then crosses during the disabled-StaticMover alarm; it remains unimplemented until that physical Everest path is verified.],
  source-evidence: evidence(
    path: [Celeste/BounceBlock.cs],
    symbol: [BounceBlock.Update],
    snippet: raw(block: true, lang: "cs", "MoveStaticMovers(Position - oldPosition);\nCollidable = true;\nstate = States.Waiting;\nAlarm.Set(this, 0.35f, EnableStaticMovers);"),
    note: [方块先移动仍 disabled 的附属物并恢复 body，0.35 秒 Alarm 之后才重新启用 StaticMover；若方块在 Alarm 窗口再次移动，就可能把附属尖刺留在偏移位置。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [BounceBlockSnapshot.static_movers_enabled / advance_bounce_blocks]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [core_block_moves_disabled_spikes_before_the_reform_alarm_reenables_them / core_block_candidate_clears_source_body_before_reform_blocked_check]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.18.3-core-block-entity-displacement.ts; scripts/e2e-real/scenarios/reform-parts.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs; Celeste/CassetteBlock.cs], symbol: [entity-4.18.3-core-block-entity-displacement; TECH_ENTITY_4_18_3_CORE_BLOCK_ENTITY_DISPLACEMENT; SnapshotCapture.Capture; CassetteBlock.Update], note: [2026-07-28 隔离真实 Everest run `2026-07-28T17-36-38.746Z-100144-9db259b6-006d-47f8-94c5-9c22eb6ab9c1` 在 frame 35 破碎后至 frame 220 仍未观测 body reform；f131 才开始离开原位置，因而不能证明 BounceBlock 的 BlockedCheck 已获准。后续 run `2026-07-28T20-38-16.708Z-102272-918982d1-99fc-4303-aac0-2e11d3b37c1b` 使用 candidate `05068c326ae8905df434bcda5d0517188e5b6d3a`、物理 `vendor/celeste-game`、nonce `f2b659f3-3416-4383-954f-ad104c417a23` 与 child PID `84540`（ports `61774/61775`，隔离 save/tmp，cleanup 均完成）；telemetry 显示 f35 Broken、f134 timer 到零时 player 约 `(802,412)` 仍占 source x=`750..814`，故 BlockedCheck 拒绝，semantic gate 在 comparator 前报 body 未重组。最终实体 E2E run `2026-07-28T20-52-00.803Z-92716-aef55361-1f6a-4c87-ab77-f5da99211bb5`（manifest `.tmp/e2e-runs/2026-07-28T20-52-00.803Z-92716-aef55361-1f6a-4c87-ab77-f5da99211bb5/manifest.json`）以 candidate `fb76c7d84fa16f7dff9867c8a1e2585aca1c71f6` 在物理 `vendor/celeste-game` 执行，nonce `916eee4d-a519-4929-a018-ce834489e937`、child PID `64480`、ports `52735/52736`，隔离 save/tmp 与 cleanup 均完成；仍在 comparator 前以 `block body did not reform while its StaticMover remained disabled` 失败。测得该 candidate 的 x=704 reset body 撞 playground base 的 x=`688..712` tile column，最近 engine-clear 目标是 x=712；当前 candidate 将 body、attached spike 与一像素右侧 jump-thru 一致右移 8px，保留 player path，并要求真实运行无固定帧依赖地观测 source clear → collidable body/spike disabled → alarm 内 second native bounce → reenabled displaced spike。该 +8px candidate 的真实 E2E run `2026-07-28T21-28-06.380Z-111408-73701773-4ff3-4291-99a7-9fed7e079ced`（manifest `.tmp/e2e-runs/2026-07-28T21-28-06.380Z-111408-73701773-4ff3-4291-99a7-9fed7e079ced/manifest.json`）以 candidate `8703a9e6072ed74e04e71ce6c113917b238931bb` 在物理 `vendor/celeste-game` 执行，nonce `b470fa1a-13c3-4a89-80c3-24f662d0803f`、child PID `110292`、ports `52396/52397`，隔离 save/tmp 与 cleanup 均完成；reform/StaticMover semantic 已通过，但仍在 comparator 前以 `block did not begin its native second bounce while the StaticMover alarm was active` 失败。故证据未闭合，保持 candidate/unimplemented。CassetteBlock 的 `BlockedCheck`/最多 4px actor wiggle 是同类实体更新先判阻塞、后启用碰撞的对照：不能用人工状态开关替代真实离开 source 的输入路径。]),
)
