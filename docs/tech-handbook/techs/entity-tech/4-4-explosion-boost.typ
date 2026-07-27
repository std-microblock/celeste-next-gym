#import "../../template.typ": tech, evidence

#tech(
  id: "4.4",
  title-zh: "爆炸加速",
  title-en: "Explosion Boost",
  status: "unimplemented",
  description-zh: [河豚、Bumper 或复活 Seeker 的爆炸会推开玩家；按住爆炸推动方向还能额外增加约 50 水平速度。],
  description-en: [Explosion knockback from puffers, bumpers, or reviving seekers gains roughly 50 extra horizontal speed when the matching direction is held.],
  source-evidence: evidence(path: [Source/Player/Player.cs], symbol: [Player.ExplodeLaunch], note: [爆炸方向存在水平分量时，同向输入会立刻把 Speed.X 乘以 1.2；不同向则短暂保存 explodeLaunchBoostSpeed，允许下一更新帧补领。]),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [explode_launch / step]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [bumper_enters_launch_and_applies_same_direction_boost_immediately / bumper_defers_horizontal_boost_when_input_is_not_held]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [entity-4.4-explosion-boost], note: [待隔离 Celeste 实测九字段。]),
)
