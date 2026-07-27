#import "../../template.typ": tech, evidence

#tech(
  id: "3.7.4",
  title-zh: "7 格跳",
  title-en: "7jump",
  status: "implemented",
  description-zh: [以满空速接双 Cornerboost，把 6 格跳再延长一格；通常要求更低的墙上起跳位置。],
  description-en: [A full-speed double cornerboost extends the six-tile setup to a seven-tile crossing.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.NormalUpdate; Player.ClimbJump; Player.Jump; Player.OnCollideH; Player.Update],
    snippet: raw(block: true, lang: "cs", "Speed.X += JumpHBoost * moveX;\n...\nwallSpeedRetained = Speed.X;\nwallSpeedRetentionTimer = WallSpeedRetentionTime;\n...\nSpeed.X = wallSpeedRetained;"),
    note: [满空速约 90 时连续两次 ClimbJump 依次叠加 40；第二次撞角把约 165.66666 存入 retention，清角后仍保持足够速度跨越从墙角到目标前缘的 56 像素。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/playground.rs],
    symbol: [climb_jump; update_wall_speed_retention; mechanics_playground],
    note: [模拟器使用正常地面加速和跳跃建立 90 空速，再执行两次真实攀跳；Playground 为本技巧提供墙角至目标前缘恰好 7 格的独立几何。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [seven_jump_lands_on_a_target_seven_tiles_from_the_double_cornerboost_wall],
    note: [独立测试从地面零速起跑，第 44 状态帧以 90 空速进入双 Cornerboost，体力依次降至 82.5 和 55，保存 165.66666，并在第 80 状态帧落到墙角外 56 像素的目标平台；全部固体均按 Celeste 的 8px tile 网格对齐。],
  ),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [seven-jump],
    note: [真实 Playground 共 121 个状态帧，从 168/120 地面零速起跑；第 44 状态帧以 90 空速抵达墙角，第 45、46 状态帧连续攀跳后体力为 82.5、55，并保存 165.66666；第 49 状态帧清角返还 161.33331，第 80 状态帧在中心 x=294 落到前缘 x=296、距墙角 x=240 恰好 56 像素的目标平台。九类核心字段逐帧一致，最大位置误差 0、速度误差 0.000001。],
  ),
  candidate-e2e: none,
)
