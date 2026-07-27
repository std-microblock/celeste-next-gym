#import "../../template.typ": tech, evidence

#tech(
  id: "4.5",
  title-zh: "实体踩跳",
  title-en: "Fish / Iceball / Oshiro / Seeker / Snowball Jump",
  status: "unimplemented",
  description-zh: [踩到河豚、冰球、Oshiro、Seeker 或雪球顶部时按住跳会显著提高反弹高度；反弹也能取消冲刺并保留水平动量。],
  description-en: [Holding jump on top-bounces from fish, ice balls, Oshiro, seekers, or snowballs increases height and can cancel a dash while preserving horizontal speed.],
  source-evidence: evidence(path: [Source/Player/Player.cs; Source/FireBall.cs], symbol: [Player.Bounce / FireBall.OnBounce], note: [Ice Ball 顶踩调用 Player.Bounce：切回 Normal、补充冲刺与体力、清除 dashAttackTimer、设置 -140 垂直速度和 0.2 秒可变跳；水平速度不被覆盖。]),
  rust-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/sim.rs], symbol: [EntityKind.IceBall / bounce / interact]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [ice_ball_bounce_cancels_dash_and_preserves_horizontal_speed / playground_ice_ball_dash_bounce_scenario_reaches_the_top_collider]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [entity-4.5-iceball-jump], note: [静止 cold FireBall 场景，待隔离 Celeste 实测九字段。]),
)
