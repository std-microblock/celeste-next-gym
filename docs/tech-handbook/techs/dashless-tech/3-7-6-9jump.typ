#import "../../template.typ": tech, evidence

#tech(
  id: "3.7.6",
  title-zh: "9 格跳",
  title-en: "9jump",
  status: "implemented",
  description-zh: [沿用 8 格跳序列，但依靠更有利的水平子像素使距离延长到 9 格。],
  description-en: [The eight-tile sequence reaches nine tiles when started with a sufficiently favorable horizontal subpixel.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.NormalUpdate; Player.ClimbJump; Player.Jump; Player.OnCollideH; Player.Update],
    snippet: raw(block: true, lang: "cs", "Speed.X += JumpHBoost * moveX;\n...\nwallSpeedRetained = Speed.X;\nwallSpeedRetentionTimer = WallSpeedRetentionTime;\n...\nSpeed.X = wallSpeedRetained;"),
    note: [同一组 JumpHBoost 与墙速保存规则会把有利接墙时序产生的 190.33347 水平速度保留下来，足以延伸至 72 像素。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/playground.rs],
    symbol: [climb_jump; update_wall_speed_retention; mechanics_playground],
    note: [模拟器以正常输入形成有利接墙时序；独立 Playground booth 的墙角与目标前缘严格相距 72 像素。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [nine_jump_lands_nine_tiles_away_only_with_the_favorable_timing],
    note: [候选在第 6、7、8 输入帧执行三次攀跳，第 45 状态帧落到 9 格目标；将整套输入延后一帧的对照明确无法落上平台。],
  ),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [nine-jump],
    note: [真实 Playground 共 121 个状态帧，从 755/120 地面零速起跑；第 6、7、8 输入帧执行三次攀跳，体力依次为 82.5、55、27.5，第 8 状态帧保存 190.33347 水平速度，第 45 状态帧在中心 x=837 落到前缘 x=840、距墙角 x=768 严格 72 像素的目标平台。九类核心字段逐帧一致，最大位置误差 0、速度误差 0.000001。],
  ),
  candidate-e2e: none,
)
