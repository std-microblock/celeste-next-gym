#import "../../template.typ": tech, evidence

#tech(
  id: "4.13",
  title-zh: "Feather Super",
  title-en: "Feather Super",
  status: "unimplemented",
  description-zh: [羽毛状态下沿地面水平移动并起跳，会退出羽毛并形成长跳；它与普通 Super 的底层机制不同。],
  description-en: [Jumping from horizontal grounded feather movement exits feather state into a long jump unrelated to normal superdash logic.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.StarFlyUpdate / Player.Jump],
    note: [StarFly 的 jump cancel 位于移动速度更新之后；只有非 transforming、Jump.Pressed 且 OnGround(3) 才调用普通 Jump 并回 Normal。普通 Jump 保留当帧水平羽毛速度，再叠加 moveX 的 40。],
    snippet: raw(block: true, lang: "cs", "if (Input.Jump.Pressed && OnGround(3)) {\n    Jump();\n    return StNormal;\n}\n...\nSpeed.X += JumpHBoost * moveX;\nSpeed.Y = JumpSpeed;"),
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [star_fly_update / grounded_at_offset], note: [先把飞行速度向当帧上限逼近，再以 3px 地面探测退出 StarFly，写入 -105 纵速并叠加 40 水平 Jump boost。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; scripts/e2e-real/test/production-registry.test.ts], symbol: [feather_super_jumps_from_grounded_horizontal_starfly_speed / keeps every feather proof in an independently named map part]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.13-feather-super.ts], symbol: [entity-4.13-feather-super], note: [独立地面 Feather MapPart 已断言 StarFly → Normal、273.333/-105 与离地；等待真实 Everest 九字段差分。]),
)
