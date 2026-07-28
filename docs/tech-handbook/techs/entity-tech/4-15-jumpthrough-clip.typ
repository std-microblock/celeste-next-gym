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
    symbol: [downward_solid_push_uses_player_squish_target_to_clip_through_jump_thru / ordinary_downward_solid_push_moves_the_actor_without_squish],
    note: [一项回归锁定向下夹压穿过 JumpThru 且不死亡，另一项防止普通非 squish 推动退化。],
  ),
  e2e-evidence: none,
  candidate-e2e: evidence(
    path: [scripts/e2e-real/scenarios/core-heart-squish-parts.ts / scripts/e2e-real/scenarios/playground/entity-4.15-jumpthrough-clip.ts],
    symbol: [tech.entity-4.15-jumpthrough-clip / entity-4.15-jumpthrough-clip],
    note: [独立 MapPart 使用向下 ZipMover 与 JumpThru；候选守卫要求玩家先到平台上，再被 pusher 夹穿到平台下且全程存活。真实 Everest 尚待 FIFO 锁内调参与采集。],
  ),
)
