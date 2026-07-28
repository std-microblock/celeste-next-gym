#import "../../template.typ": tech, evidence

#tech(
  id: "4.19",
  title-zh: "Seeker Bounce",
  title-en: "Seeker Bounce",
  status: "unimplemented",
  description-zh: [Seeker 撞墙后的短暂状态允许玩家从侧面反弹，恢复冲刺并获得接近 Hyper 的水平速度，纵向速度取决于接触角度。],
  description-en: [A seeker briefly becomes side-bounceable after hitting a wall, refilling dash and granting near-hyper horizontal speed with angle-dependent vertical motion.],
  source-evidence: evidence(
    path: [Source/Seeker.cs / Source/Player/Player.cs],
    symbol: [Seeker.SlammedIntoWall / Seeker.OnAttackPlayer / Player.PointBounce],
    snippet: raw(block: true, lang: "cs", "Speed.X = Math.Sign(Speed.X) * -100f;\nSpeed.Y *= 0.4f;\nState.State = StStunned;\n...\nplayer.PointBounce(Center);"),
    note: [Seeker 以至少 100 水平速度撞墙后先反向为 100、纵速乘 0.4 并进入 Stunned；Stunned 每帧以 150 接近零并在 0.8 秒后回 Idle。此时侧面接触才调用 PointBounce、恢复冲刺／体力，并令 Seeker 以 100 远离玩家。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs / crates/celeste-physics/src/sim.rs], symbol: [SeekerSnapshot / advance_seekers / move_seeker_axis]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [seeker_attack_wall_collision_enters_stunned_with_source_speeds_and_timer / stunned_seeker_side_contact_point_bounces_player_and_recoils_at_one_hundred / seeker_stunned_coroutine_returns_idle_and_split_simulation_is_composable]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.19-seeker-bounce.ts], symbol: [entity-4.19-seeker-bounce], note: [最终真实候选形成 PointBounce 语义，但第 36 帧先差：Rust 位置 (214,481)、速度 (90,135)，Everest 位置 (214,483)、速度 (90,-140)；最大位置／速度误差 74／300，保持未实现。]),
)
