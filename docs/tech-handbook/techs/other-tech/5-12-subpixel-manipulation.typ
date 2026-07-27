#import "../../template.typ": tech, evidence

#tech(
  id: "5.12",
  title-zh: "子像素控制",
  title-en: "Subpixel Manipulation",
  status: "implemented",
  description-zh: [通过交替输入控制 Actor 的 movementCounter 小数余数；尚未跨过半像素时位置不变，跨过 ties-to-even 舍入边界时移动一个整数像素并保留带符号余数。],
  description-en: [Alternating inputs deliberately tune Actor movementCounter: position stays fixed below the half-pixel rounding boundary, then advances one integer pixel while retaining the signed remainder.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Monocle/Actor.cs],
    symbol: [Player.Update; Actor.MoveH; Actor.MoveHExact],
    snippet: raw(block: true, lang: "cs", "MoveH(Speed.X * Engine.DeltaTime, onCollideH);\n...\nmovementCounter.X += moveH;\nint move = (int) Math.Round(movementCounter.X, MidpointRounding.ToEven);\nmovementCounter.X -= move;\nMoveHExact(move, onCollide);"),
    note: [Player 每帧把 Speed.X×DeltaTime 交给 Actor；Actor 先累加不可见小数，再以 bankers rounding 取整数像素并把差值留到后续帧。碰撞逐像素处理并会清除对应轴余数，因此相同整数位置可携带不同后续轨迹。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [PlayerSnapshot.movement_remainder; move_axis_amount], note: [Rust 快照保存双轴余数，move_axis_amount 使用 round_ties_even 后逐像素碰撞，保持分段模拟闭包。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [subpixel_manipulation_accumulates_air_control_until_a_pixel_crossing], note: [五帧交替输入依次断言 X 余数 0.180556、0.361113，随后整数位置从 160 变 161、余数变为 -0.458331，并覆盖其余核心字段。]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.12-subpixel-manipulation.ts], symbol: [other-5.12-subpixel-manipulation; verifySubpixelManipulation], note: [独立 MapPart 读取真实 Everest movementCounter，逐帧验证两次纯子像素累积和第三次半像素越界；九类核心字段比较容差不超过 0.01。]),
  candidate-e2e: none,
)
