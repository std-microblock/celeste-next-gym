#import "../../template.typ": tech, evidence

#tech(
  id: "2.1",
  title-zh: "弹簧取消",
  title-en: "Spring Cancel",
  status: "unimplemented",
  description-zh: [接触普通或侧向弹簧后立刻冲刺，会用冲刺速度覆盖弹簧赋予的动量；输入也可以预缓冲。],
  description-en: [Dashing immediately after a normal or sideways spring replaces the spring momentum, and the dash input may be buffered.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste.Spring.OnCollide],
    symbol: [Player.SuperBounce; Player.SideBounce; Spring.OnCollide],
    snippet: raw(block: true, lang: "cs", "// floor spring\nplayer.SuperBounce(Top); // Speed = 0 / -185, refill dash + stamina\n// wall spring\nplayer.SideBounce(dir, faceX, CenterY); // Speed = ±240 / -140\n// buffered dash is consumed by NormalUpdate on the next frame"),
    note: [弹簧碰撞先回填冲刺与体力并回到 Normal；普通 Dash 虚拟按键的 0.08 秒缓冲仍保留，所以接触后的下一帧可用冲刺覆盖弹簧速度。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/sim.rs], symbol: [EntityKind.Spring; super_bounce; side_bounce; PlayerSnapshot.dash_buffer_timer]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [spring_cancel_uses_the_buffered_dash_after_the_spring_refills_it; wall_spring_uses_source_side_bounce_speed_and_force_move]),
  e2e-evidence: none,
  candidate-e2e: "dash-spring-cancel",
)
