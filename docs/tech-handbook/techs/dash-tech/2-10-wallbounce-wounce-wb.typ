#import "../../template.typ": tech, evidence

#tech(
  id: "2.10",
  title-zh: "Wallbounce 墙反",
  title-en: "Wallbounce (wounce, wb)",
  status: "implemented",
  description-zh: [向上冲刺贴近墙面时，在 Dash Attack 窗口内按跳可向外弹出，获得约 170 水平速度和 160 向上速度。],
  description-en: [Jumping beside a wall during an upward dash attack launches a wallbounce at about 170 horizontal and 160 upward speed.],
  source-evidence: evidence(path: [Source/Player/Player.cs], symbol: [Player.DashUpdate; Player.SuperWallJump], note: [上冲期间墙探测命中后设置 170 水平、-160 垂直与 0.25 s 可变跳窗口。]),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [dash_update; super_wall_jump]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [wallbounce_sets_super_wall_jump_speed_and_var_window]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [wallbounce], note: [真实关键帧 -170/-160，九类字段 max speed error 0.000038。]),
  candidate-e2e: none,
)
