#import "../../template.typ": tech, evidence

#tech(
  id: "2.8.3",
  title-zh: "延迟 Ultra",
  title-en: "Delayed Ultra",
  status: "implemented",
  description-zh: [只要最后冲刺方向仍是向下斜，玩家可先在别处保存该方向，延迟到之后落地时才触发 1.2 倍加速和蹲伏。],
  description-en: [A stored down-diagonal last-dash direction can trigger the ultra multiplier and crouch on a later landing far from the original dash.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.OnCollideV],
    snippet: raw(block: true, lang: "cs", "if (DashDir.X != 0 && DashDir.Y > 0 && Speed.Y > 0) {\n    DashDir.X = Math.Sign(DashDir.X);\n    DashDir.Y = 0;\n    Speed.Y = 0;\n    Speed.X *= DodgeSlideSpeedMult;\n    Ducking = true;\n}"),
    note: [落地分支只检查保存的 DashDir 和当前下降速度，不要求仍处于 Dash 状态；因此冲刺结束后较晚落地仍可触发倍率。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [PlayerSnapshot.dash_dir; move_axis_amount]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [delayed_ultra_lands_after_dash_state_and_still_multiplies_speed], note: [回归先确认玩家已离开 Dash，再以经过空中摩擦的落地前速度计算并断言 1.2 倍结果。]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [dash-delayed-ultra; verifyDelayedUltra], note: [真实 37 帧场景确认玩家先离开 Dash，随后空中水平减速一帧并在落地时把该速度精确乘 1.2；九类核心字段最大位置与速度误差均为 0。]),
  candidate-e2e: none,
)
