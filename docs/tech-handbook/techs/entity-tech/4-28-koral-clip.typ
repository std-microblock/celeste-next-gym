#import "../../template.typ": tech, evidence

#tech(
  id: "4.28",
  title-zh: "Koral Clip",
  title-en: "Koral Clip",
  status: "unimplemented",
  description-zh: [移动实体把玩家或投掷物压进屏幕边界后再反向移动时，防卡墙逻辑可能把对象传送到实体移动方向一侧。],
  description-en: [If a moving solid clips the player or a holdable against a screen edge and then reverses, escape logic can teleport it to the solid's moving side.],
  source-evidence: evidence(
    path: [Source/Actor.cs / Source/Solid.cs / Source/Player/Player.cs / Source/TempleGate.cs],
    symbol: [Actor.TrySquishWiggle / Solid.MoveHExact / Solid.MoveVExact / Player.OnSquish],
    snippet: raw(block: true, lang: "cs", "if (!CollideCheck<Solid>(data.TargetPosition + vector2))\n{\n    Position = data.TargetPosition + vector2;\n    return true;\n}"),
    note: [移动 Solid 推压 Actor 时会携带原始 TargetPosition 进入 squish 回调。TrySquishWiggle 先搜当前位置附近，再搜目标位置附近并可直接搬移对象；Player 另有 TargetPosition 特判。当前模拟器只有简化 moving-solid 推动，不具备这套目标位置回退、屏幕边界碰撞和 TempleGate 反向移动链，无法源码一致地实现 Koral Clip。],
  ),
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
