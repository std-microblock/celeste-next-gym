#import "../../template.typ": tech, evidence

#tech(
  id: "4.18.2.1",
  title-zh: "Cassoosted Fuper",
  title-en: "Cassoosted Fuper",
  status: "implemented",
  description-zh: [在执行 Feather Super 的同一时机获得 Cassette Reform Boost，把两种技巧的长跳和纵向加速组合起来。],
  description-en: [CassetteBlock reformation writes an upward LiftSpeed before the grounded StarFly jump; Player.Jump assigns JumpSpeed and then consumes that lift, combining the Feather Super's horizontal launch with the Cassette reform boost.],
  source-evidence: evidence(
    path: [Celeste/CassetteBlock.cs; Source/Player/Player.cs],
    symbol: [CassetteBlock.Update; Player.StarFlyUpdate; Player.Jump],
    snippet: raw(block: true, lang: "cs", "if (Input.Jump.Pressed && OnGround(3)) {\n    Jump();\n    return StNormal;\n}\n...\nCollidable = true;\nEnableStaticMovers();\nShiftSize(-1);"),
    note: [CassetteBlock 在 Player 前完成 wiggle、恢复碰撞与 1px 上移，Solid.MoveV 把向上 LiftSpeed 写给玩家；随后 StarFly grounded Jump 先设 JumpSpeed，再消费该 LiftBoost。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [step; advance_cassette_blocks; advance_cassette_manager; star_fly_update], note: [每帧先运行 cassette block 的 reform MoveV，再执行 Player/StarFly；manager 仍在 Player 后写下一帧 Activated，因此 fresh fixture 保留 f28 activation、f29 reform/Fuper 的真实顺序。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [cassoosted_fuper_combines_grounded_starfly_jump_and_same_frame_reform; cassoosted_fuper_fixture_consumes_reform_lift_on_next_player_update], note: [fixture 锁定 tempo-three cassette 在 f29 恢复碰撞、写入约 `-60` LiftSpeed，并由同帧 grounded Feather Super 在 JumpSpeed 后消费，得到 `(273.33334,-165)`；同时覆盖 reform wiggle、carry 与消失 block 的时序。]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.18.2.1-cassoosted-fuper.ts; scripts/e2e-real-collector.mjs], symbol: [entity-4.18.2.1-cassoosted-fuper; compareRealTrace], note: [2026-07-28 隔离物理 Everest run `2026-07-28T19-36-00.169Z-74496-16bb7873-487e-43b6-934d-90635fa125c5` 完成 vendor 安装校验、nonce/精确 child PID 认证及受控清理。101 个状态逐帧比较 position、speed、state、facing、dashes、stamina、grounded、ducking、death；position 与 speed 最大误差均为 0。frame 29 同时观察 index-0 `(304,493)`/`collidable=true` 和 Normal Feather Super `(273.3333,-164.99988)`。]),
  candidate-e2e: none,
)
