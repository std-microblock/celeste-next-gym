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
    note: [Throw 先调用 Holdable.Release 并施加反冲，再清空 Holding；水平 DashUpdate 仍只在 jumpGraceTimer 有效时进入 SuperJump。Release 同时设置 0.1 秒 gravityTimer 与 CannotHold，Core Hyper 后必须等锁定结束才能在空中重抓。],
    snippet: raw(block: true, lang: "cs", "Holding.Release(Vector2.UnitX * (int)Facing);\nSpeed.X += ThrowRecoil * -(int)Facing;\nHolding = null;\n...\nif (DashDir.Y == 0 && CanUnDuck && Input.Jump.Pressed && jumpGraceTimer > 0) {\n    SuperJump();\n    return StNormal;\n}\n...\nHolder = null;\ngravityTimer = .1f;\ncannotHoldTimer = cannotHoldDelay;"),
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs / crates/celeste-physics/src/types.rs],
    symbol: [release_theo / try_pickup_theo / advance_bounce_blocks / PlayerSnapshot.theo_cannot_hold_timer],
    note: [Rust 复用热态 BounceBlock 的发射 lift 与 jump grace，按原版先释放 Theo、施加反冲、建立 CannotHold，再执行蹲伏 SuperJump；Pickup 碰撞在锁定结束后可于空中重新进入 StPickup。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [holdable_core_hyper_releases_during_grace_then_regrabs_after_cannot_hold],
    note: [回归同时锁定 release recoil、0.1 秒 CannotHold、Core Hyper 速度和空中重抓顺序。],
  ),
  e2e-evidence: none,
  candidate-e2e: evidence(
    path: [scripts/e2e-real/scenarios/core-heart-squish-parts.ts / scripts/e2e-real/scenarios/playground/entity-4.11-holdable-core-hyper.ts],
    symbol: [tech.entity-4.11-holdable-core-hyper / entity-4.11-holdable-core-hyper],
    note: [独立 MapPart 放置标准热态 BounceBlock 与 Theo；候选场景验证抓取、释放、CannotHold、Core Hyper 和空中重抓，真实 Everest 尚待 FIFO 锁内调参与采集。],
  ),
)
