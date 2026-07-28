#import "../../template.typ": tech, evidence

#tech(
  id: "4.12",
  title-zh: "Featherboost 羽毛加速",
  title-en: "Featherboost",
  status: "implemented",
  description-zh: [进入羽毛移动状态的第一帧输入斜方向，会获得额外的初始速度。],
  description-en: [Holding a diagonal direction on the first feather-movement frame grants an initial speed boost.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.StarFlyCoroutine],
    note: [变身动画结束且速度归零后，协程额外等待 0.1 秒；随后只读取该帧 Input.Aim，并直接乘以 StarFlyStartSpeed 250。斜向 Aim 已归一化，所以两轴为约 ±176.7767。],
    snippet: raw(block: true, lang: "cs", "while (Speed != Vector2.Zero) yield return null;\nyield return .1f;\nvar dir = Input.Aim.Value;\nif (dir == Vector2.Zero) dir = Vector2.UnitX * (int)Facing;\nSpeed = dir * StarFlyStartSpeed;\nstarFlyLastDir = dir;"),
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [begin_star_fly / star_fly_update], note: [变身倒计时结束的同一模拟帧读取 input_vector，空输入回退 Facing，并以 250 设置首个活动帧速度。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; scripts/e2e-real/test/production-registry.test.ts], symbol: [featherboost_uses_the_first_live_diagonal_for_the_250_start_speed / keeps every feather proof in an independently named map part]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.12-featherboost.ts], symbol: [entity-4.12-featherboost], note: [独立 Feather MapPart 的真实首活动帧得到归一化斜向速度 `(176.77669,-176.77669)`，总模长 250。46 个状态的 position 最大误差 0、speed 最大误差 0.000076，其余核心字段逐帧一致。]),
  candidate-e2e: none,
)
