#import "../../template.typ": tech, evidence

#tech(
  id: "5.3",
  title-zh: "卡带方块抬升",
  title-en: "Cassette Raise",
  status: "unimplemented",
  description-zh: [玩家留在卡带方块内部反复经历开关，会让方块每个周期上移 2 像素；残留色带没有实体碰撞。],
  description-en: [The pre-toggle and activation phases each shift a cassette block by one pixel; overlap resolution can repeat the two-pixel raise while the changing visual height is not an extra Solid. Rust models both phases, but real Everest evidence and video are still pending.],
  source-evidence: evidence(
    path: [Celeste/CassetteBlockManager.cs; Celeste/CassetteBlock.cs],
    symbol: [CassetteBlockManager.AdvanceMusic; CassetteBlock.WillToggle; CassetteBlock.Update; CassetteBlock.ShiftSize],
    snippet: raw(block: true, lang: "cs", "if ((beatIndex + 1) % 8 == 0) SetWillActivate(next);\n...\npublic void WillToggle() { ShiftSize(Collidable ? 1 : -1); }\n...\nCollidable = true;\nShiftSize(-1);\n...\nprivate void ShiftSize(int amount) {\n    MoveV(amount);\n    blockHeight -= amount;\n}"),
    note: [目标颜色先在切换前一拍执行一次 `ShiftSize(-1)`，正式启用且 `BlockedCheck/TryActorWiggleUp` 通过后再上移一像素。`blockHeight` 只控制侧面图像缩放，真正碰撞仍由 Solid 的 Position/Collider 决定。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/types.rs], symbol: [advance_cassette_manager; advance_cassette_blocks; CassetteManagerSnapshot], note: [manager 以源码常量 `355/(678π)` 推进六teenth beat；beat 7 先 WillToggle 1px，beat 8 只改 Activated，下一实体帧再 BlockedCheck 并 MoveV 1px。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [cassette_raise_uses_separate_will_toggle_and_activation_pixels], note: [回归逐帧断言两颜色在预切换时同处中间高度，下一拍后的实体帧才分别到 active source Y 与 inactive source Y+2。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.3-cassette-raise.ts; scripts/e2e-real/scenarios/cassette-spinner-parts.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs], symbol: [other-5.3-cassette-raise; TECH_OTHER_5_3_CASSETTE_RAISE; SnapshotCapture.Capture], note: [独立双颜色 CassetteBlock MapPart 已保留。collector 新增逐帧 cassette runtime 后，候选直接寻找 index-0 的非碰撞 Y=494 WillToggle 帧及随后 Collidable 的 Y=493 reform 帧，取代无法证明该阶段的 player-only Y 轨迹；尚未重跑真实 Everest，故保持 candidate。]),
)
