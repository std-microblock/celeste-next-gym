#import "../../template.typ": tech, evidence

#tech(
  id: "4.14",
  title-zh: "Heart Ultra",
  title-en: "Heart Ultras",
  status: "unimplemented",
  description-zh: [在收集水晶心的同一帧向下斜冲刺，可利用心脏补冲和状态中断形成 Ultra；可先耗尽冲刺把输入变成缓冲。],
  description-en: [Down-diagonal dashing on the heart-collection frame combines the refill and interruption into an ultra, with dash exhaustion enabling a buffered setup.],
  source-evidence: evidence(
    path: [Celeste/HeartGem.cs (v1.4.0 decompile); Source/Player/Player.cs],
    symbol: [HeartGem.OnPlayer / HeartGem.CollectRoutine / Player.StartDash],
    note: [HeartGem 只在 DashAttacking 时开始 Collect；协程先跨一帧，再触发 0.2 秒 raw-time Freeze，随后把 Engine.TimeRate 写为 0.5。Heart Ultra 依赖收集回调、补充冲刺与玩家 Dash 状态中断在同一更新序列中的精确先后。],
    snippet: raw(block: true, lang: "cs", "if (player.DashAttacking) {\n    Collect(player);\n    return;\n}\n...\nyield return null;\nCeleste.Freeze(.2f);\nyield return null;\nEngine.TimeRate = .5f;"),
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/map.rs / crates/celeste-physics/src/types.rs / crates/celeste-physics/src/sim.rs],
    symbol: [EntityKind.HeartGem / HeartGemSnapshot / advance_heart_gems],
    note: [HeartGem 已进入 BinaryPacker schema 与 runtime；DashAttacking 接触标记 collected，协程状态按一帧 yield、0.2 秒全局 Freeze、TimeRate=0.5 前进，非冲刺接触沿 PointBounce 控制路径处理。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/map.rs / crates/celeste-physics/src/sim.rs],
    symbol: [vanilla_heart_gem_round_trips_through_celeste_binary / heart_gem_collect_yields_then_freezes_before_setting_half_time_rate / heart_gem_point_bounces_a_non_dash_attacking_player],
  ),
  e2e-evidence: none,
  candidate-e2e: evidence(
    path: [scripts/e2e-real/scenarios/core-heart-squish-parts.ts / scripts/e2e-real/scenarios/playground/entity-4.14-heart-ultra.ts],
    symbol: [tech.entity-4.14-heart-ultra / entity-4.14-heart-ultra],
    note: [独立 MapPart 以 vanilla `blackGem` 编码水晶心，Collector 直接扫描未注册 Tracker 的 HeartGem。真实 trace 在 frame 5 收集、frame 7 进入 0.2 秒 Freeze、frame 20 得到首个 0.495833 TimeRate，语义链完整；但 frame 21 起原版用 `Engine.DeltaTime = RawDeltaTime * TimeRate` 每帧移动约 3px，Rust 固定 DT 每帧移动 6px，九字段首差因此保留未实现。],
  ),
)
