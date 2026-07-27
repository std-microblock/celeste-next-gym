#import "../../template.typ": tech, evidence

#tech(
  id: "2.8",
  title-zh: "Ultra 超冲",
  title-en: "Ultradash (Ultra)",
  status: "implemented",
  description-zh: [高速移动时向下斜冲刺，并在冲刺结束后接触地面，会把水平速度乘以 1.2；落地和起跳时序决定速度能否保留。],
  description-en: [A down-diagonal dash performed above 170 horizontal speed can apply a 1.2 multiplier on landing when dash-end and jump timing preserve it.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.OnCollideV],
    snippet: raw(block: true, lang: "cs", "if (DashDir.X != 0 && DashDir.Y > 0 && Speed.Y > 0) {\n    DashDir.X = Math.Sign(DashDir.X);\n    DashDir.Y = 0;\n    Speed.Y = 0;\n    Speed.X *= DodgeSlideSpeedMult; // 1.2\n    Ducking = true;\n}"),
    note: [向下斜冲触地时，OnCollideV 把最后冲刺方向改为水平、清零纵速、将当前水平速度乘 1.2 并切换蹲伏碰撞箱；该判定读取保留的 DashDir，因此也构成 Delayed Ultra 的核心。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [move_axis_amount; PlayerSnapshot.dash_dir]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [ultradash_landing_applies_the_source_one_point_two_multiplier], note: [回归锁定落地前水平速度超过 170，并断言触地帧精确乘以 1.2、纵速归零且进入 ducking。]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [dash-ultra; verifyUltra], note: [真实 13 帧场景在 Dash 状态内落地，精确得到 240/√2×1.2 的水平速度、纵速归零并把 DashDir 展平为 1/0；九类核心字段最大位置与速度误差均为 0。]),
  candidate-e2e: none,
)
