#import "../../template.typ": tech, evidence

#tech(
  id: "4.18.2.1",
  title-zh: "Cassoosted Fuper",
  title-en: "Cassoosted Fuper",
  status: "unimplemented",
  description-zh: [在执行 Feather Super 的同一时机获得 Cassette Reform Boost，把两种技巧的长跳和纵向加速组合起来。],
  description-en: [This composition requires both the CassetteBlock reform wiggle/ShiftSize lifecycle and a grounded StarFly jump on the same frame. Both Rust components are implemented, but the verdict remains unimplemented until one real Everest trace proves the composition and records video.],
  source-evidence: evidence(
    path: [Celeste/CassetteBlock.cs; Source/Player/Player.cs],
    symbol: [CassetteBlock.Update; Player.StarFlyUpdate; Player.Jump],
    snippet: raw(block: true, lang: "cs", "if (Input.Jump.Pressed && OnGround(3)) {\n    Jump();\n    return StNormal;\n}\n...\nCollidable = true;\nEnableStaticMovers();\nShiftSize(-1);"),
    note: [Player 先以 StarFly 水平速度执行 grounded Jump 并退出状态；随后房间实体更新让 CassetteBlock 完成 wiggle、恢复碰撞与 1px 上移。必须在同一真实帧链观察两部分，不能只凭 Feather Super 晋级。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [star_fly_update; advance_post_player_entities; advance_cassette_blocks], note: [Player/StarFly 按实体顺序先更新，CassetteBlock 后更新；快照同时保留 StarFly 与 cassette runtime，分段模拟仍可组合。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [cassoosted_fuper_combines_grounded_starfly_jump_and_same_frame_reform], note: [单帧回归同时断言 StarFly→Normal 的 `(273.33334,-105)` 与 Cassette 恢复碰撞、上移到 source position、玩家额外纵向位移。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.18.2.1-cassoosted-fuper.ts; scripts/e2e-real/scenarios/cassette-spinner-parts.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs], symbol: [entity-4.18.2.1-cassoosted-fuper; TECH_ENTITY_4_18_2_1_CASSOOSTED_FUPER; SnapshotCapture.Capture], note: [2026-07-28 隔离真实 Everest run `2026-07-28T13-41-44.489Z-90784-8038eac1-ccd2-4e7b-99a6-226711fa7273` 完成 nonce/PID 认证及受控清理；持续 right aim 后仍未观测到要求的 `(273.33334,-105)` grounded Feather Super（semantic gate fuper=-1），不转正。]),
)
