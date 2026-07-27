#import "../../template.typ": tech, evidence

#tech(
  id: "1.6",
  title-zh: "方向性尖刺",
  title-en: "Directional Spikes",
  status: "implemented",
  description-zh: [玩家沿尖刺指向移动时，与尖刺接触不会死亡；判定依赖接触方向和当帧速度。],
  description-en: [Spikes do not kill the player when contact occurs while moving in the direction the spikes point.],
  source-evidence: evidence(
    path: [Celeste.Spikes.OnCollide],
    symbol: [Spikes.OnCollide],
    snippet: raw(block: true, lang: "cs", "case Directions.Up:\n    if (player.Speed.Y >= 0f && player.Bottom <= Bottom) player.Die(-Vector2.UnitY);\n    break;\ncase Directions.Left:\n    if (player.Speed.X >= 0f) player.Die(-Vector2.UnitX);\n    break;"),
    note: [每个方向只在玩家速度朝向刺尖或静止时致死；沿刺尖指向远离危险面移动时接触安全。下刺与右刺使用对应的反向不等式。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [spike_is_lethal; interact]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [directional_spikes_only_kill_motion_into_their_points; upward_motion_flush_with_directional_spikes_applies_gravity_on_frame_one]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [mechanics-directional-spikes-away; mechanics-directional-spikes-into], note: [同一上刺面分别以 -60 向上远离和 60 向下撞入；安全场景全程存活，危险场景 frame 1 死亡，九类核心字段的最大位置与速度误差均为 0。]),
  candidate-e2e: none,
)
