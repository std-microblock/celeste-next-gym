#import "../../template.typ": tech, evidence

#tech(
  id: "4.28",
  title-zh: "Koral Clip",
  title-en: "Koral Clip",
  status: "unimplemented",
  description-zh: [移动实体把玩家或投掷物压进屏幕边界后再反向移动时，防卡墙逻辑可能把对象传送到实体移动方向一侧。],
  description-en: [If a moving solid clips the player or a holdable against a screen edge and then reverses, escape logic can teleport it to the solid's moving side.],
  source-evidence: evidence(
    path: [Source/Actor.cs / Source/Solid.cs / Source/Player/Player.cs / Source/TempleGate.cs],
    symbol: [Actor.TrySquishWiggle / Solid.MoveHExact / Solid.MoveVExact / Player.OnSquish],
    snippet: raw(block: true, lang: "cs", "if (!CollideCheck<Solid>(data.TargetPosition + vector2))\n{\n    Position = data.TargetPosition + vector2;\n    return true;\n}"),
    note: [TempleGate.SetHeight 从 0 关闭时先临时扩成 64px，再以一次 MoveVExact 移动 closedHeight；Actor 的 TargetPosition 因而是完整推压目标。TrySquishWiggle 按源码顺序先搜当前位置 ±3，再搜 TargetPosition ±3 并可直接搬移；Player 在此之前还会尝试蹲下后的当前位置与目标点。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs / crates/celeste-physics/src/sim.rs], symbol: [TempleGateSnapshot / close_temple_gate / squish_wiggle_candidate / squish_player]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs / crates/celeste-physics/src/map.rs], symbol: [close_behind_player_gate_uses_target_position_fallback_to_clip_theo / player_squish_tries_ducked_target_position_before_actor_wiggles / failed_gate_squish_kills_theo_with_player_and_removes_glider / vanilla_close_behind_player_temple_gate_round_trips_through_celeste_binary]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.28-koral-clip.ts], symbol: [entity-4.28-koral-clip], note: [独立候选使用真实 CloseBehindPlayerAlways TempleGate、JumpThru 与 Theo，已能验证关门后实体碰撞体；现有 Everest 九字段采集不暴露 Theo 位置，尚不能证明 TargetPosition 回退后的真实传送坐标，故保持未实现。]),
)
