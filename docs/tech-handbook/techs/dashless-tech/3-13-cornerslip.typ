#import "../../template.typ": tech, evidence

#tech(
  id: "3.13",
  title-zh: "Cornerslip 擦角补冲",
  title-en: "Cornerslip",
  status: "implemented",
  description-zh: [以不发生实体碰撞的方式擦过地面墙角，可以恢复冲刺且不重置纵向速度，并获得土狼时间。],
  description-en: [Grazing a floor corner without a solid collision can refill the dash, preserve vertical speed, and grant coyote time.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste.DreamBlock],
    symbol: [Player.Update; DreamBlock.Added],
    snippet: raw(block: true, lang: "cs", "if (onGround) jumpGraceTimer = JumpGraceTime;\n...\nelse if (onGround && CollideCheck<Solid>(Position + Vector2.UnitY))\n    RefillDash();\n...\nMoveH(Speed.X * Engine.DeltaTime, onCollideH);\nMoveV(Speed.Y * Engine.DeltaTime, onCollideV);"),
    note: [Update 帧首的一像素 ground probe 先赋予土狼时间并补冲，随后才 MoveH 再 MoveV。未解锁梦冲时 DreamBlock 仍继承并保持 Solid 碰撞；擦过角点而两轴均未撞停时，纵速得以保留。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/sim.rs], symbol: [EntityKind.DreamBlock; update_grounded_resources; move_h; move_v]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [cornerslip_over_disabled_dream_block_refills_without_vertical_collision]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/cornerslip.ts], symbol: [cornerslip; verifyCornerslip], note: [独立 disabled DreamBlock MapPart 共 7 个真实状态；state 1 到达 35/41，速度保持 -90/60、冲刺补为 1、仍未接地且 jumpGraceTimer 大于 0.09。九类字段逐帧一致，位置与速度最大误差均为 0。]),
  candidate-e2e: none,
)
