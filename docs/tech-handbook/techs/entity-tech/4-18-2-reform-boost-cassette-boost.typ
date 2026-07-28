#import "../../template.typ": tech, evidence

#tech(
  id: "4.18.2",
  title-zh: "Reform Boost 重组加速",
  title-en: "Reform Boost (Cassette Boost)",
  status: "implemented",
  description-zh: [卡带方块在玩家靠近顶部时重组，会把玩家向上校正到顶面；同步起跳可把这段瞬移转成巨大 liftboost。],
  description-en: [CassetteBlock checks BlockedCheck, may wiggle an overlapping actor up by at most four pixels, then enables collision and shifts its Solid upward by one pixel. Rust reproduces the beat/index lifecycle, and the final record-mode Everest run matches all nine core fields while producing a trustworthy video and poster.],
  source-evidence: evidence(
    path: [Celeste/CassetteBlock.cs; Celeste/CassetteBlockManager.cs],
    symbol: [CassetteBlock.Update / TryActorWiggleUp / ShiftSize],
    snippet: raw(block: true, lang: "cs", "if (!flag) {\n    Collidable = true;\n    EnableStaticMovers();\n    ShiftSize(-1);\n}\n...\nfor (int i = 1; i <= 4; i++)\n    if (!actor.CollideCheck<Solid>(actor.Position - Vector2.UnitY * i))\n        actor.Position -= Vector2.UnitY * i;"),
    note: [正式启用前先做 BlockedCheck，最多向上 wiggle 4px；通过后恢复 Solid/StaticMover 并再上移 1px。完整技巧还依赖 manager 的全局 beat、颜色 index 与 group 同步。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/types.rs], symbol: [EntityKind.CassetteBlock; advance_cassette_blocks; advance_cassette_manager; CassetteBlockSnapshot], note: [快照持久化 beat/index、Activated/Collidable 与两阶段 Position；重组按源码先尝试玩家 1..4px 上移，再恢复 Solid 并用 -60px/s lift 执行 1px MoveV。custom cassette music 首帧只创建 sfx、不调用 AdvanceMusic 的分支也保存在 startup_music_pending。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [cassette_reform_wiggles_player_four_pixels_then_carries_one; cassette_raise_uses_separate_will_toggle_and_activation_pixels; fresh_custom_cassette_manager_skips_music_advance_on_its_first_update], note: [回归固定验证 4px wiggle 后再被 Solid 上移 1px、预切换与正式激活分帧，并锁定 custom manager 首帧不推进音乐。]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.18.2-reform-boost-cassette-boost.ts; scripts/e2e-real/scenarios/cassette-spinner-parts.ts], symbol: [entity-4.18.2-reform-boost-cassette-boost; TECH_ENTITY_4_18_2_REFORM_BOOST], note: [独立双颜色 CassetteBlock MapPart 在最终合并 HEAD 的正式 record mode 采集 101 个状态与 130 个反射字段；九个必需字段逐帧一致，最大 position/speed 误差均为 0，最终 Y=493。presentation manifest 以 final_state_tail_presented 正常完成，并生成绑定本次 trace 的 H.264 MP4（150 帧、2.5 秒）、poster 与 artifacts manifest。]),
  candidate-e2e: none,
)
