#import "../../template.typ": tech, evidence

#tech(
  id: "1.12",
  title-zh: "蹲伏抗风",
  title-en: "Wind Resistance",
  status: "implemented",
  description-zh: [在有风区域保持蹲伏时，风不会推动玩家。],
  description-en: [Crouching prevents wind from pushing the player.],
  source-evidence: evidence(path: [Source/Player/Player.cs], symbol: [Player.WindMove], note: [水平风在接地蹲伏时乘 DuckWindMult=0，并保留墙体遮挡检查。]),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [advance_wind_controller; apply_wind_movement]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [grounded_ducking_blocks_horizontal_wind_movement]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [playground-wind-ground-ducking], note: [45 帧强风中接地蹲伏位置和速度保持不变；九类字段最大误差 0。]),
  candidate-e2e: none,
)
