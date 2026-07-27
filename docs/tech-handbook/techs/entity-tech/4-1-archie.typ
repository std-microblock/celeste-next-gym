#import "../../template.typ": tech, evidence

#tech(
  id: "4.1",
  title-zh: "Archie 泡泡抬升",
  title-en: "Archie",
  status: "implemented",
  description-zh: [以蹲伏碰撞箱进入泡泡时，泡泡按矮碰撞箱居中，会让玩家位置比正常进入高 2 像素。],
  description-en: [Entering a bubble with the crouched hitbox centers the smaller body two pixels higher than a normal entry.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.BoostUpdate / Player.BoostEnd],
    note: [泡泡每帧和退出时都用 `boostTarget - Collider.Center` 定位。蹲伏碰撞箱中心比站立碰撞箱低 2 像素，因此同一泡泡中心会把蹲伏玩家放到高 2 像素的位置。],
    snippet: raw(block: true, lang: "cs", "Vector2 target = Calc.Approach(\n    ExactPosition,\n    boostTarget - Collider.Center + Input.Aim.Value * 3f,\n    80f * Engine.DeltaTime\n);\nMoveToX(target.X);\nMoveToY(target.Y);\n\n// BoostEnd\nVector2 snap = (boostTarget - Collider.Center).Floor();"),
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [boost_update / snap_to_boost_target],
    snippet: raw(block: true, lang: "rust", "let center_offset_y = if p.ducking { 3.0 } else { 5.5 };\nlet target = Vec2::new(\n    p.boost_target.x.floor(),\n    (p.boost_target.y + center_offset_y).floor(),\n);"),
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [archie_centers_the_duck_hitbox_two_pixels_above_a_normal_boost],
  ),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [entity-4.1-archie],
    note: [独立泡泡场景共 37 个状态；position/speed 最大误差均为 0，state、facing、dashes、stamina、grounded、ducking、death 全部一致。],
  ),
  candidate-e2e: none,
)
