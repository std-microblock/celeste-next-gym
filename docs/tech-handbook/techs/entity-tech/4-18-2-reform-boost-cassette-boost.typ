#import "../../template.typ": tech, evidence

#tech(
  id: "4.18.2",
  title-zh: "Reform Boost 重组加速",
  title-en: "Reform Boost (Cassette Boost)",
  status: "unimplemented",
  description-zh: [卡带方块在玩家靠近顶部时重组，会把玩家向上校正到顶面；同步起跳可把这段瞬移转成巨大 liftboost。],
  description-en: [CassetteBlock checks BlockedCheck, may wiggle an overlapping actor up by at most four pixels, then enables collision and shifts its Solid upward by one pixel. The Rust beat/index and reform lifecycle is implemented; the verdict remains unimplemented until the candidate has a real Everest nine-field trace and video.],
  source-evidence: evidence(
    path: [Celeste/CassetteBlock.cs; Celeste/CassetteBlockManager.cs],
    symbol: [CassetteBlock.Update / TryActorWiggleUp / ShiftSize],
    snippet: raw(block: true, lang: "cs", "if (!flag) {\n    Collidable = true;\n    EnableStaticMovers();\n    ShiftSize(-1);\n}\n...\nfor (int i = 1; i <= 4; i++)\n    if (!actor.CollideCheck<Solid>(actor.Position - Vector2.UnitY * i))\n        actor.Position -= Vector2.UnitY * i;"),
    note: [正式启用前先做 BlockedCheck，最多向上 wiggle 4px；通过后恢复 Solid/StaticMover 并再上移 1px。完整技巧还依赖 manager 的全局 beat、颜色 index 与 group 同步。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/types.rs], symbol: [EntityKind.CassetteBlock; advance_cassette_blocks; advance_cassette_manager; CassetteBlockSnapshot], note: [快照持久化 beat/index、Activated/Collidable 与两阶段 Position；重组按源码先尝试玩家 1..4px 上移，再恢复 Solid 并用 -60px/s lift 执行 1px MoveV。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [cassette_reform_wiggles_player_four_pixels_then_carries_one; cassette_raise_uses_separate_will_toggle_and_activation_pixels], note: [回归固定验证 4px wiggle 后再被 Solid 上移 1px，并验证预切换与正式激活不在同一帧。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.18.2-reform-boost-cassette-boost.ts; scripts/e2e-real/scenarios/cassette-spinner-parts.ts], symbol: [entity-4.18.2-reform-boost-cassette-boost; TECH_ENTITY_4_18_2_REFORM_BOOST], note: [独立双颜色 CassetteBlock MapPart 已通过 canonical fixture 编译；真实 Everest 九字段与视频等待 FIFO 锁内采集。]),
)
