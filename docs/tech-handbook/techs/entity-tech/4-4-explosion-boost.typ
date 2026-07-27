#import "../../template.typ": tech, evidence

#tech(
  id: "4.4",
  title-zh: "爆炸加速",
  title-en: "Explosion Boost",
  status: "implemented",
  description-zh: [河豚、Bumper 或复活 Seeker 的爆炸会推开玩家；按住爆炸推动方向还能额外增加约 50 水平速度。],
  description-en: [Explosion knockback from puffers, bumpers, or reviving seekers gains roughly 50 extra horizontal speed when the matching direction is held.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.ExplodeLaunch],
    snippet: raw(block: true, lang: "cs", "Speed = 280f * vector;\nif (Speed.Y <= 50f)\n    Speed.Y = Math.Min(-150f, Speed.Y);\nif (Speed.X != 0f) {\n    if (Input.MoveX.Value == Math.Sign(Speed.X))\n        Speed.X *= 1.2f;\n    else {\n        explodeLaunchBoostTimer = 0.01f;\n        explodeLaunchBoostSpeed = Speed.X * 1.2f;\n    }\n}"),
    note: [先从爆炸方向生成 280 速度并把向上分量至少钳到 -150；若水平输入同向，当帧把 X 乘 1.2，不同向则只保存 0.01 秒的补领窗口。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [explode_launch / step]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [bumper_enters_launch_and_applies_same_direction_boost_immediately / bumper_defers_horizontal_boost_when_input_is_not_held]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [entity-4.4-explosion-boost], note: [真实 Bumper 左侧发射且持续按左；状态帧 1 立即进入 Launch，速度从 -280 加成到 -336，Y 为 -150。31 个状态九类字段逐帧一致，position 与 speed 最大误差均为 0。]),
  candidate-e2e: none,
)
