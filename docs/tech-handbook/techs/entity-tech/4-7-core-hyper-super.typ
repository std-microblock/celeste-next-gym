#import "../../template.typ": tech, evidence

#tech(
  id: "4.7",
  title-zh: "Core 方块 Hyper／Super",
  title-en: "Core Hyper/Super",
  status: "implemented",
  description-zh: [Core 方块发射结束时会留下土狼时间和巨大 liftboost；在窗口内做 Hyper 或 Super 可叠加两者获得高速。],
  description-en: [Core blocks leave coyote time and strong liftboost at launch end, which a hyper or super can combine into very high speed.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Celeste/BounceBlock.cs / Celeste/Platform.cs / Celeste/Solid.cs],
    symbol: [BounceBlock.Update / BounceBlock.ShakeOffPlayer / Player.SuperJump / Player.LiftBoost / Platform.MoveTo],
    note: [热态 BounceBlock 先沿 `-bounceDir * 10` 蓄力，再沿 `bounceDir * 24` 发射；结束时以最高 200 的 bounceLift 把玩家切回 Normal 并开启 0.1 秒 jump grace。SuperJump 先叠加裁剪到 Y=-130 的 LiftBoost，蹲伏版本随后应用 X=1.25、Y=0.5 倍率，因此垂直发射可分别得到 `(260,-235)` 与 `(325,-117.5)`。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/map.rs / crates/celeste-physics/src/types.rs / crates/celeste-physics/src/sim.rs],
    symbol: [EntityKind.BounceBlock / BounceBlockSnapshot / advance_bounce_blocks / move_bounce_block_to],
    note: [实现保存 Waiting、WindingUp、Bouncing、BounceEnd、Broken/respawn 的原版状态、Platform movementCounter 与 LiftSpeed，并按 Player 先更新、BounceBlock 后 carry/push 和 ShakeOffPlayer 的实体顺序执行。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/map.rs / crates/celeste-physics/src/sim.rs],
    symbol: [vanilla_bounce_block_round_trips_through_celeste_binary / hot_bounce_block_shakes_off_player_with_source_lift_and_jump_grace / bounce_block_runtime_keeps_split_simulation_composable / broken_bounce_block_reforms_after_source_respawn_timer / playground_hot_bounce_block_grace_adds_core_super_lift / playground_hot_bounce_block_grace_adds_core_hyper_lift],
  ),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [entity-4.7-core-super / entity-4.7-core-hyper],
    note: [两个独立 Playground 场景均从标准热态 bounceBlock 顶面开始，等待真实方块发射后在留下的 grace 内执行水平 Super 或 Demo Hyper。语义守卫确认发射速度 Y=-200、jumpGraceTimer=0.1、lastLiftSpeed.Y=-200；每项各 39 个状态，position/speed 最大误差均为 0，其余七类字段逐帧一致，关键速度分别为 `(260,-235)` 与 `(325,-117.5)`。],
  ),
  candidate-e2e: none,
)
