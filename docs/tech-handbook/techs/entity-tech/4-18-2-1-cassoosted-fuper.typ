#import "../../template.typ": tech, evidence

#tech(
  id: "4.18.2.1",
  title-zh: "Cassoosted Fuper",
  title-en: "Cassoosted Fuper",
  status: "unimplemented",
  description-zh: [在执行 Feather Super 的同一时机获得 Cassette Reform Boost，把两种技巧的长跳和纵向加速组合起来。],
  description-en: [CassetteBlock reformation writes an upward LiftSpeed in the entity phase; the next grounded StarFly jump calls Player.Jump and consumes that retained lift after assigning JumpSpeed. The Rust timing is covered locally, but the verdict remains unimplemented until a fresh real Everest nine-field comparison and video certify the composition.],
  source-evidence: evidence(
    path: [Celeste/CassetteBlock.cs; Source/Player/Player.cs],
    symbol: [CassetteBlock.Update; Player.StarFlyUpdate; Player.Jump],
    snippet: raw(block: true, lang: "cs", "if (Input.Jump.Pressed && OnGround(3)) {\n    Jump();\n    return StNormal;\n}\n...\nCollidable = true;\nEnableStaticMovers();\nShiftSize(-1);"),
    note: [Player 先以 StarFly 水平速度执行 grounded Jump 并退出状态；随后房间实体更新让 CassetteBlock 完成 wiggle、恢复碰撞与 1px 上移。必须在同一真实帧链观察两部分，不能只凭 Feather Super 晋级。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [star_fly_update; advance_post_player_entities; advance_cassette_blocks], note: [Player/StarFly 按实体顺序先更新，CassetteBlock 后更新；快照同时保留 StarFly 与 cassette runtime，分段模拟仍可组合。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [cassoosted_fuper_combines_grounded_starfly_jump_and_same_frame_reform; cassoosted_fuper_fixture_consumes_reform_lift_on_next_player_update], note: [候选 fixture 从 fresh Feather/manager 运行，锁定 tempo-three cassette 的 f28 reform 写入约 `-60` LiftSpeed，以及 f29 grounded Feather Super 先设 JumpSpeed 后得到 `(273.33334,-165)`；该一帧间隔保留 Player→实体的源码更新顺序。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.18.2.1-cassoosted-fuper.ts; scripts/e2e-real/scenarios/cassette-spinner-parts.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs], symbol: [entity-4.18.2.1-cassoosted-fuper; TECH_ENTITY_4_18_2_1_CASSOOSTED_FUPER; SnapshotCapture.Capture], note: [2026-07-28 隔离真实 Everest runs `2026-07-28T14-10-09.822Z-74888-88bf63e0-49b5-4033-b930-27c41693f908` 与 `2026-07-28T19-31-41.609Z-87164-814c0586-8987-4555-9090-39ca9b063a74` 均完成物理 vendor 安装校验、nonce/精确 child PID 认证及受控清理。frame 29 的 index-0 已恢复到 `(304,493)` 且 `collidable=true`；Feather 退出为 Normal，速度为 `(273.3333,-164.99988)`，与保留 LiftBoost 的源码顺序一致。但最新九字段 Rust 回放在 frame 28 仍提前上移 1px，最大位置误差 3px，未满足 0.01 门槛，故保持未实现。]),
)
