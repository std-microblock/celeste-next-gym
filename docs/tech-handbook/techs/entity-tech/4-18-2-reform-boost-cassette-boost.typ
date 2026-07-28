#import "../../template.typ": tech, evidence

#tech(
  id: "4.18.2",
  title-zh: "Reform Boost 重组加速",
  title-en: "Reform Boost (Cassette Boost)",
  status: "unimplemented",
  description-zh: [卡带方块在玩家靠近顶部时重组，会把玩家向上校正到顶面；同步起跳可把这段瞬移转成巨大 liftboost。],
  description-en: [CassetteBlock checks BlockedCheck, may wiggle an overlapping actor up by at most four pixels, then enables collision and shifts its Solid upward by one pixel. Rust has no CassetteBlockManager beat/index/group lifecycle, so this remains unimplemented.],
  source-evidence: evidence(
    path: [Celeste/CassetteBlock.cs; Celeste/CassetteBlockManager.cs],
    symbol: [CassetteBlock.Update / TryActorWiggleUp / ShiftSize],
    snippet: raw(block: true, lang: "cs", "if (!BlockedCheck()) {\n    Collidable = true;\n    EnableStaticMovers();\n    ShiftSize(-1);\n}\n...\nfor (int i = 1; i <= 4; i++) actor.MoveVExact(-1);"),
    note: [正式启用前先做 BlockedCheck，最多向上 wiggle 4px；通过后恢复 Solid/StaticMover 并再上移 1px。完整技巧还依赖 manager 的全局 beat、颜色 index 与 group 同步。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/sim.rs], symbol: [EntityKind / advance_move_blocks], note: [现有 MoveBlock/BounceBlock runtime 不能替代 CassetteBlockManager 的全局节拍与分组激活。]),
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
