#import "../../template.typ": tech, evidence

#tech(
  id: "4.8",
  title-zh: "延迟方块加速",
  title-en: "Delayed Blockboost",
  status: "implemented",
  description-zh: [离开移动方块后的约 9 帧内，即使改在另一面墙上起跳，仍可把之前保存的 blockboost 应用到跳跃。],
  description-en: [For roughly nine frames after leaving a moving block, a wall jump from another surface can still apply the stored blockboost.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Celeste/Actor.cs / Celeste/ZipMover.cs],
    symbol: [Actor.LiftSpeed / Actor.Update / Player.WallJump / Player.LiftBoost / ZipMover.Sequence],
    note: [Actor 会把最后一次非零 LiftSpeed 保存 0.16 秒；WallJump 仅在保存值为零时才改取当前墙的 LiftSpeed，随后把裁剪后的 LiftBoost 叠加到 `(130,-105)`。因此离开 ZipMover 后可在另一静态墙上延迟消费原方块速度。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/types.rs / crates/celeste-physics/src/sim.rs],
    symbol: [PlayerSnapshot.player_on_ground / advance_zip_movers / tick_lift_speed / normal_update],
    note: [快照分别保存 Player.Update 时刻的内部 onGround 与帧末公开几何 OnGround，使 Player 先更新、ZipMover 后 carry/push 的实体顺序在分段模拟中保持闭包；保留的 LiftSpeed 继续由墙跳消费。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [delayed_blockboost_uses_zip_lift_on_a_later_static_wall_jump / delayed_climb_wall_jump_uses_retained_lift_speed / zip_mover_runs_after_player_and_matches_real_vertical_push_order],
  ),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [entity-4.8-delayed-blockboost],
    note: [Playground 中从标准原版 ZipMover 向右离开，普通 0.1 秒 jump grace 归零后，在隔离的静态墙上执行 WallJump。语义守卫确认终态速度为 `(-130,-230.828)`，而不是普通墙跳的 `(-130,-105)`；26 个状态的 position 最大误差 0、speed 最大误差 0.000001，其余七类字段逐帧一致。],
  ),
  candidate-e2e: none,
)
