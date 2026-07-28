#import "../../template.typ": tech, evidence

#tech(
  id: "4.15.1",
  title-zh: "Feather Clip",
  title-en: "Feather Clip",
  status: "implemented",
  description-zh: [羽毛状态即将结束前接触单向平台，可让状态切换后的玩家出现在平台另一侧。],
  description-en: [Touching a jumpthrough just before feather state expires can place the player through the platform as the hitbox changes.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [starFlyHitbox / StarFlyUpdate / StarFlyEnd],
    note: [StarFly collider 为 8×8、相对位置 (-4,-10)，普通 collider 为 8×11、相对位置 (-4,-11)。飞行计时在移动之后递减并返回 Normal；状态回调才恢复普通 collider，所以临界帧能先以短 collider 穿过单向平台顶面。],
    snippet: raw(block: true, lang: "cs", "normalHitbox = new Hitbox(8, 11, -4, -11);\nstarFlyHitbox = new Hitbox(8, 8, -4, -10);\n...\nstarFlyTimer -= Engine.DeltaTime;\nif (starFlyTimer <= 0) return StNormal;\n...\nCollider = normalHitbox;"),
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [star_fly_rect / move_axis_amount / star_fly_update / end_star_fly], note: [下落移动先以 StarFly rect 做 JumpThru 顶面判定；同帧 timer 到期后才恢复普通 hitbox，保留源码的碰撞—状态切换顺序。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; scripts/e2e-real/test/production-registry.test.ts], symbol: [feather_clip_exits_below_the_jumpthrough_top / keeps every feather proof in an independently named map part]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.15.1-feather-clip.ts], symbol: [entity-4.15.1-feather-clip], note: [独立 Feather + 16px JumpThru MapPart 真实观察到飞行移动先以 8×8 collider 越过平台顶面，随后计时到期恢复 Normal；退出帧位于平台下方、未 grounded 且未死亡。181 个状态的 position 最大误差 0、speed 最大误差 0.000015，其余核心字段逐帧一致。]),
  candidate-e2e: none,
)
