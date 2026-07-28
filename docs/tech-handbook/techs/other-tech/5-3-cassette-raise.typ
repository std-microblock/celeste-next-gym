#import "../../template.typ": tech, evidence

#tech(
  id: "5.3",
  title-zh: "卡带方块抬升",
  title-en: "Cassette Raise",
  status: "unimplemented",
  description-zh: [玩家留在卡带方块内部反复经历开关，会让方块每个周期上移 2 像素；残留色带没有实体碰撞。],
  description-en: [The pre-toggle and activation phases each shift a cassette block by one pixel; overlap resolution can repeat the two-pixel raise while the changing visual height is not an extra Solid. Rust has no cassette entity, color groups, or global beat clock, so this remains unimplemented.],
  source-evidence: evidence(
    path: [Celeste/CassetteBlockManager.cs; Celeste/CassetteBlock.cs],
    symbol: [CassetteBlockManager.AdvanceMusic; CassetteBlock.WillToggle; CassetteBlock.Update; CassetteBlock.ShiftSize],
    snippet: raw(block: true, lang: "cs", "if ((beatIndex + 1) % 8 == 0) SetWillActivate(next);\n...\npublic void WillToggle() { ShiftSize(Collidable ? 1 : -1); }\n...\nCollidable = true;\nShiftSize(-1);\n...\nprivate void ShiftSize(int amount) {\n    MoveV(amount);\n    blockHeight -= amount;\n}"),
    note: [目标颜色先在切换前一拍执行一次 `ShiftSize(-1)`，正式启用且 `BlockedCheck/TryActorWiggleUp` 通过后再上移一像素。`blockHeight` 只控制侧面图像缩放，真正碰撞仍由 Solid 的 Position/Collider 决定。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/sim.rs], symbol: [EntityKind; advance_moving_solids], note: [实体枚举没有 CassetteBlock，模拟器也没有 index/color、1/6 秒 beat、八拍切换、BlockedCheck 或分阶段 `ShiftSize`；MovingSolid 不能替代该全局实体状态机。]),
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
