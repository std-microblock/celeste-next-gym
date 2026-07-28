#import "../../template.typ": tech, evidence

#tech(
  id: "4.15",
  title-zh: "单向平台夹穿",
  title-en: "Jumpthrough Clip",
  status: "unimplemented",
  description-zh: [玩家被实体向下挤压到单向平台时，防挤压处理会把玩家推到平台下方。],
  description-en: [When crushed downward against a jumpthrough, squish handling can push the player through to its underside.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste/Actor.cs (v1.4.0 decompile)],
    symbol: [Player.OnSquish / Actor.TrySquishWiggle],
    note: [Player.OnSquish 先临时改成 duck collider，并分别测试当前位置与 data.TargetPosition；仍被 Solid 挤压时才调用 Actor.TrySquishWiggle。当前 Rust 没有 CollisionData、Pusher/TargetPosition、移动实体压迫与 squish-wiggle 搜索，所以普通 JumpThru 碰撞不足以证明该技巧。],
    snippet: raw(block: true, lang: "cs", "if (!Ducking) {\n    Ducking = true;\n    data.Pusher.Collidable = true;\n    if (!CollideCheck<Solid>()) return;\n    Position = data.TargetPosition;\n    if (!CollideCheck<Solid>()) return;\n    Position = was;\n}\nif (!TrySquishWiggle(data)) Die(Vector2.Zero);"),
  ),
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
