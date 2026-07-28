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
    note: [Player.OnSquish 先临时改成 duck collider，并分别测试当前位置与 data.TargetPosition；仍被 Solid 挤压时才调用 Actor.TrySquishWiggle。JumpThru 参与 Actor 向下移动碰撞，却不是 OnSquish 的 Solid 检查对象，因此 TargetPosition 可落到平台下方。],
    snippet: raw(block: true, lang: "cs", "if (!Ducking) {\n    Ducking = true;\n    data.Pusher.Collidable = true;\n    if (!CollideCheck<Solid>()) return;\n    Position = data.TargetPosition;\n    if (!CollideCheck<Solid>()) return;\n    Position = was;\n}\nif (!TrySquishWiggle(data)) Die(Vector2.Zero);"),
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [move_solid_y / player_on_squish / try_squish_wiggle],
    note: [碰撞感知 Solid pusher 保留 Pusher 与 TargetPosition；OnSquish 严格按临时 duck、当前位置 Solid 检查、TargetPosition Solid 检查、3×3 wiggle 搜索执行，JumpThru 只参与 Actor 的向下移动阻挡。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [downward_solid_push_uses_player_squish_target_to_clip_through_jump_thru / squish_wiggle_disables_the_pusher_after_target_position_checks / ordinary_downward_solid_push_moves_the_actor_without_squish / zip_mover_runtime_invokes_target_position_jump_thru_clip],
    note: [单步回归锁定 TargetPosition 夹穿、关闭 pusher 后的 wiggle、以及普通非 squish 推动；完整 runtime 回归再由 ZipMover 的实际 outbound phase 触发 Solid push，验证 TargetPosition 穿过 JumpThru 且玩家存活。],
  ),
  e2e-evidence: none,
  candidate-e2e: evidence(
    path: [scripts/e2e-real/scenarios/core-heart-squish-parts.ts / scripts/e2e-real/scenarios/playground/entity-4.15-jumpthrough-clip.ts],
    symbol: [tech.entity-4.15-jumpthrough-clip / entity-4.15-jumpthrough-clip],
    note: [独立 MapPart 使用向下 ZipMover 与 JumpThru。首轮玩家直到 mover 到达后才落在其顶面；一次输入时序修正后，玩家从右缘只移动到 x=655，碰撞体仍与 mover 相交并持续作为 rider 被 carry 到 y=408，未落到 JumpThru、未触发 OnSquish，故不以候选近似宣称实现。],
  ),
)
