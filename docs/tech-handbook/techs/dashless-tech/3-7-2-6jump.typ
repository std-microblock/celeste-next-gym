#import "../../template.typ": tech, evidence

#tech(
  id: "3.7.2",
  title-zh: "6 格跳",
  title-en: "6jump",
  status: "unimplemented",
  description-zh: [满空中横移速度下在墙顶做一次 Cornerboost，把约 90 的速度再增加 40，以跨越 6 格缺口。],
  description-en: [At full air-strafe speed, one top-corner boost adds about 40 speed and crosses a six-tile gap.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.Jump; Player.OnCollideH; Player.Update],
    snippet: raw(block: true, lang: "cs", "private const float JumpHBoost = 40f;\n...\nSpeed.X += JumpHBoost * moveX;\n...\nwallSpeedRetained = Speed.X;\nwallSpeedRetentionTimer = WallSpeedRetentionTime;\n...\nSpeed.X = wallSpeedRetained;"),
    note: [满空中横移速度约为 90；墙唇攀跳先把 40 加到该速度，再由水平碰撞保存并在清角后返还约 130，形成 6 格跳所需的额外距离。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [climb_jump; update_wall_speed_retention; JUMP_H_BOOST],
    note: [模拟器不注入 retention，而是从 90 初速经真实攀跳、撞墙、清角顺序自然生成并返还 130。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [six_jump_uses_a_full_speed_cornerboost_to_reach_six_tile_landing],
    note: [独立几何的两平台墙角相距 48 像素；测试验证首帧保存 130，并在一次 Cornerboost 后越过 6 格落到目标平台。],
  ),
  e2e-evidence: none,
  candidate-e2e: "six-jump",
)
