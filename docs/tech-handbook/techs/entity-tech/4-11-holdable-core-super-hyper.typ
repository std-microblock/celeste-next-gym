#import "../../template.typ": tech, evidence

#tech(
  id: "4.11",
  title-zh: "携物 Core Hyper／Super",
  title-en: "Holdable Core Super/Hyper",
  status: "unimplemented",
  description-zh: [携物抓在 Core 方块附近，利用方块土狼时间先丢出物品，再做 Core Super／Hyper 并在空中重新抓回。],
  description-en: [Throw a held item during core-block coyote time, perform the core super or hyper, and regrab the item in flight.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste/Holdable.cs (v1.4.0 decompile)],
    symbol: [Player.Throw / Player.DashUpdate / Holdable.Release],
    note: [Throw 先调用 Holdable.Release 并施加反冲，再清空 Holding；水平 DashUpdate 仍只在 jumpGraceTimer 有效时进入 SuperJump。当前 Rust 快照没有 Holding、持有物释放轨迹、cannotHoldTimer 与空中重抓碰撞，因而不能用普通输入重现“先丢、Core Super/Hyper、再抓回”的完整链路。],
    snippet: raw(block: true, lang: "cs", "Holding.Release(Vector2.UnitX * (int)Facing);\nSpeed.X += ThrowRecoil * -(int)Facing;\nHolding = null;\n...\nif (DashDir.Y == 0 && CanUnDuck && Input.Jump.Pressed && jumpGraceTimer > 0) {\n    SuperJump();\n    return StNormal;\n}\n...\nHolder = null;\ngravityTimer = .1f;\ncannotHoldTimer = cannotHoldDelay;"),
  ),
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
