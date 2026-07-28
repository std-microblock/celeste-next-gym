#import "../../template.typ": tech, evidence

#tech(
  id: "4.19",
  title-zh: "Seeker Bounce",
  title-en: "Seeker Bounce",
  status: "unimplemented",
  description-zh: [Seeker 撞墙后的短暂状态允许玩家从侧面反弹，恢复冲刺并获得接近 Hyper 的水平速度，纵向速度取决于接触角度。],
  description-en: [A seeker briefly becomes side-bounceable after hitting a wall, refilling dash and granting near-hyper horizontal speed with angle-dependent vertical motion.],
  source-evidence: evidence(
    path: [Source/Seeker.cs / Source/Player/Player.cs],
    symbol: [Seeker.SlammedIntoWall / Seeker.OnAttackPlayer / Player.PointBounce],
    snippet: raw(block: true, lang: "cs", "Speed.X = Math.Sign(Speed.X) * -100f;\nSpeed.Y *= 0.4f;\nState.State = StStunned;\n...\nplayer.PointBounce(Center);"),
    note: [Seeker 以至少 100 水平速度撞墙后先反向为 100、纵速乘 0.4 并进入 Stunned；Stunned 接触玩家才调用 PointBounce、恢复冲刺／体力并按双方中心角度给出侧向速度。当前 Rust 只有静态 Seeker 顶踩回调，没有这段实体状态生命周期。],
  ),
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
