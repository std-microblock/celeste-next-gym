#import "../../template.typ": tech, evidence

#tech(
  id: "2.3",
  title-zh: "Hyper 冲刺跳",
  title-en: "Hyperdash (Hyper)",
  status: "implemented",
  description-zh: [向下斜冲刺接地后在冲刺结束前起跳，可获得约 325 水平初速，但跳跃高度只有普通跳的一半。],
  description-en: [Jumping out of a grounded down-diagonal dash gives about 325 horizontal speed at roughly half normal jump height.],
  source-evidence: evidence(path: [Source/Player/Player.cs], symbol: [Player.DashCoroutine; Player.SuperJump], note: [地面下斜冲刺转为蹲伏水平滑行，SuperJump 蹲伏分支乘 1.25/0.5 得到 325/-52.5。]),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [dash_update; super_jump; move_exact]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [hyperdash_applies_duck_super_multipliers]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [hyper], note: [真实关键帧速度 325/-52.5 且解除蹲伏；九类字段最大误差 0。]),
  candidate-e2e: none,
)
