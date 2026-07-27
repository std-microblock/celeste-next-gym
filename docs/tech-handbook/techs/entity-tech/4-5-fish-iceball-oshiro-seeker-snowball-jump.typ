#import "../../template.typ": tech, evidence

#tech(
  id: "4.5",
  title-zh: "实体踩跳",
  title-en: "Fish / Iceball / Oshiro / Seeker / Snowball Jump",
  status: "implemented",
  description-zh: [踩到河豚、冰球、Oshiro、Seeker 或雪球顶部时按住跳会显著提高反弹高度；反弹也能取消冲刺并保留水平动量。],
  description-en: [Holding jump on top-bounces from fish, ice balls, Oshiro, seekers, or snowballs increases height and can cancel a dash while preserving horizontal speed.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Source/FireBall.cs],
    symbol: [Player.Bounce / FireBall.OnBounce],
    snippet: raw(block: true, lang: "cs", "if (iceMode && !broken && player.Bottom <= Y + 4f && player.Speed.Y >= 0f) {\n    broken = true; Collidable = false;\n    player.Bounce((int)(Y - 2f));\n}\n...\nRefillDash(); RefillStamina();\nStateMachine.State = StNormal;\nvarJumpTimer = .2f; AutoJumpTimer = .1f;\ndashAttackTimer = 0f;\nvarJumpSpeed = Speed.Y = -140f;"),
    note: [顶部回调先检查玩家底部不低于实体顶面 4 像素且正在下落，再调用统一 Bounce。Bounce 不改 Speed.X，但切回 Normal、补满冲刺与体力、清 dash attack，并设置 -140、0.2 秒可变跳和 0.1 秒自动跳。FireBall 的 PlayerCollider 在 Player.Update 后回调，真实 trace 因此在下一玩家帧才暴露 Bounce。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/map.rs / crates/celeste-physics/src/sim.rs / crates/celeste-physics/src/types.rs], symbol: [EntityKind.IceBall / Puffer / AngryOshiro / Seeker / Snowball / pending_bounce_from_y / bounce], note: [所有顶部实体汇入同一 bounce；IceBall 先保存 pending source Y，下一 Player frame 消费，以匹配实体 PlayerCollider 的实际更新顺序并保持分段模拟闭包。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [ice_ball_bounce_cancels_dash_and_preserves_horizontal_speed / fish_oshiro_seeker_and_snowball_top_callbacks_share_player_bounce_semantics / ice_ball_pending_callback_keeps_split_simulation_composable / playground_ice_ball_dash_bounce_scenario_reaches_the_top_collider]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [entity-4.5-iceball-jump], note: [真实 cold FireBall：状态帧 5 仍以 `(169.70563,169.70563)` 保持 Dash，状态帧 6 顶踩回调后为 Normal、`(169.70563,-140)`、冲刺 1、体力 110。25 个状态九类字段逐帧一致，position 与 speed 最大误差均为 0。]),
  candidate-e2e: none,
)
