#import "../../template.typ": tech, evidence

#tech(
  id: "3.8",
  title-zh: "尖刺攀爬",
  title-en: "Spike Climb",
  status: "implemented",
  description-zh: [利用尖刺只在朝向其危险方向移动时致死的规则，以极小的离墙速度贴着尖刺墙上升并寻找跳离窗口。],
  description-en: [Tiny movement away from directional spikes can keep the player alive while climbing alongside a spiked wall.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste.Spikes.OnCollide],
    symbol: [Player.WallJump; Spikes.OnCollide],
    snippet: raw(block: true, lang: "cs", "Speed.X = WallJumpHSpeed * dir;\nSpeed.Y = JumpSpeed;\n...\ncase Directions.Left:\n    if (player.Speed.X >= 0f) player.Die(-Vector2.UnitX);\n    break;"),
    note: [WallJump 先把水平速度设为离墙方向的 130、纵速设为 -105；随后同帧左向尖刺只在 Speed.X >= 0 时致死，因此跳离带刺墙的帧是安全的。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [spike_is_lethal; wall_jump]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [spike_climb_wall_jump_sets_away_speed_before_the_spike_check]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/spike-climb.ts], symbol: [spike-climb; verifySpikeClimb], note: [独立尖刺墙 MapPart 共 17 个真实状态；state 1 为 -130/-105，持续上升且全程存活。九类字段逐帧一致，最大位置误差 0、速度误差 0。]),
  candidate-e2e: none,
)
