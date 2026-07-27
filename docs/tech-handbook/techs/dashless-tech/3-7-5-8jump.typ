#import "../../template.typ": tech, evidence

#tech(
  id: "3.7.5",
  title-zh: "8 格跳",
  title-en: "8jump",
  status: "unimplemented",
  description-zh: [地面跳加速、双 Cornerboost 和额外攀跳组合，可跨 8 格，并强烈依赖子像素位置。],
  description-en: [A ground-jump boost, double cornerboost, and extra climb jump combine into a subpixel-sensitive eight-tile crossing.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.NormalUpdate; Player.ClimbJump; Player.Jump; Player.OnCollideH; Player.Update],
    snippet: raw(block: true, lang: "cs", "Speed.X += JumpHBoost * moveX;\n...\nwallSpeedRetained = Speed.X;\nwallSpeedRetentionTimer = WallSpeedRetentionTime;\n...\nSpeed.X = wallSpeedRetained;"),
    note: [地面跳先建立超过普通跑速的水平速度，连续双 Cornerboost 与第三次 ClimbJump 再叠加水平加速；撞角保存的 179.6666 水平速度在清角后返还。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/playground.rs],
    symbol: [climb_jump; update_wall_speed_retention; mechanics_playground],
    note: [模拟器通过正常地面输入、Jump 和三次 ClimbJump 建立序列；Playground 使用全 8px 网格对齐的独立 booth，墙角到目标前缘严格为 64 像素。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [eight_jump_lands_on_a_target_eight_tiles_from_the_cornerboost_wall],
    note: [测试从地面零速起跑，第 11 状态帧到达墙角，三次攀跳把体力依次降至 82.5、55、27.5，并在第 49 状态帧落到 8 格目标平台。],
  ),
  e2e-evidence: none,
  candidate-e2e: "eight-jump",
)
