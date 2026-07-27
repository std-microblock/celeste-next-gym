#import "../../template.typ": tech, evidence

#tech(
  id: "1.1",
  title-zh: "草莓收集机制",
  title-en: "Berry Mechanics",
  status: "implemented",
  description-zh: [Follower 的 0.3 秒跟随延迟结束后，首颗草莓需要连续 9 个安全接地更新才会收集；下一颗从 -0.15 秒偏移开始，并因同帧实体更新顺序在 17 帧后收集。],
  description-en: [After the follower's 0.3-second delay, the first berry collects after nine safe-ground updates; the next starts at a -0.15-second offset and collects 17 frames later because it updates again in the same entity frame.],
  source-evidence: evidence(
    path: [Celeste.Strawberry.orig_Update; Celeste.Strawberry.orig_OnCollect],
    symbol: [Follower.FollowDelay; Strawberry.collectTimer; Player.StrawberryCollectIndex],
    snippet: raw(block: true, lang: "cs", "Follower.FollowDelay = 0.3f;\n...\nif (player.OnSafeGround) {\n    collectTimer += Engine.DeltaTime;\n    if (collectTimer > 0.15f) OnCollect();\n}\n...\nif (followIndex > 0) collectTimer = -0.15f;\n...\nplayer.StrawberryCollectIndex++;\nplayer.StrawberryCollectResetTimer = 2.5f;"),
    note: [草莓按 follower 顺序更新；前一颗收集后，后一颗同帧成为 FollowIndex 0 并立即把 -0.15 再加一次 DeltaTime，因此实测索引间隔为 17 帧。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/sim.rs], symbol: [EntityKind.Strawberry; update_strawberry_train; PlayerSnapshot.strawberry_collect_index]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [first_berry_collects_after_nine_consecutive_safe_ground_frames; later_berry_in_the_train_waits_through_the_negative_collection_offset]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [mechanics-berry-train], note: [真实双草莓场景在 follower 延迟后满足首颗 9 个安全接地更新，第二次 StrawberryCollectIndex 相隔 17 帧；65 帧九类核心字段最大位置与速度误差均为 0。]),
  candidate-e2e: none,
)
