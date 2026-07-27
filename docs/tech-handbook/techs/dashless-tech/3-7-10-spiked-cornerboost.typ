#import "../../template.typ": tech, evidence

#tech(
  id: "3.7.10",
  title-zh: "尖刺 Cornerboost",
  title-en: "Spiked Cornerboost",
  status: "implemented",
  description-zh: [在一侧或两侧带尖刺的墙角完成 Cornerboost，需要同时满足方向性尖刺的生存条件。],
  description-en: [A spiked cornerboost performs the retained-speed corner interaction while respecting directional-spike survival on adjacent faces.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste/Spikes.cs],
    symbol: [Player.ClimbJump; Player.OnCollideH; Spikes.OnCollide],
    snippet: raw(block: true, lang: "cs", "case Directions.Up:\n    if (player.Speed.Y >= 0f && player.Bottom <= base.Bottom)\n        player.Die(new Vector2(0f, -1f));\n    break;"),
    note: [上刺只有在玩家不再上升且玩家底边仍位于尖刺底边之上时致死。Cornerboost 的攀跳纵速为 -105，因此可在离开刺尖期间触发墙角碰撞并保留水平速度，而不会被方向性尖刺误杀。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [climb_jump; move_axis_amount; spike_is_lethal], note: [尖刺判定同时使用方向、速度和实体边界，复现上刺的单向致死条件。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [spiked_cornerboost_survives_only_while_rising_away_from_top_spikes]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/spiked-cornerboost.ts], symbol: [spiked-cornerboost; verifySpikedCornerboost], note: [独立带上刺墙角 MapPart 记录 21 个真实状态；state 1 体力 82.5、retained speed 130，state 5 速度 125.66666/-105，全程未死亡。九类核心字段逐帧一致，最大位置误差 0、速度误差 0.000001。]),
  candidate-e2e: none,
)
