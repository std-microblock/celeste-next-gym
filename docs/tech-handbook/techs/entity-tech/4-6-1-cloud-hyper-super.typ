#import "../../template.typ": tech, evidence

#tech(
  id: "4.6.1",
  title-zh: "云朵 Hyper／Super",
  title-en: "Cloud Hyper/Super",
  status: "implemented",
  description-zh: [在云朵向上反弹时输入 Hyper 或 Super，可把冲刺技巧的水平速度与云朵纵向加速叠加。],
  description-en: [Performing a hyper or super during a cloud's upward rebound combines dash-tech horizontal speed with the cloud's vertical boost.],
  source-evidence: evidence(
    path: [Source/Cloud.cs / Source/Player/Player.cs],
    symbol: [Cloud.Update / Player.SuperJump],
    snippet: raw(block: true, lang: "cs", "if (speed >= -100f && playerRider2?.Speed.Y >= 0f)\n    playerRider2.Speed.Y = -200f;\n...\nfloat lift = speed < 0f ? -220f : speed;\nMoveV(speed * Engine.DeltaTime, lift);"),
    note: [云回升越过 -100 阈值时先把 rider 设为 -200 纵速，上升阶段仍以 -220 lift 移动 Solid；同一窗口中的 SuperJump／Hyper 保留各自水平启动速度，因此两个分量按实体更新与玩家冲刺跳顺序叠加。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [advance_clouds / super_jump / interact], note: [独立 CloudSnapshot 保存相位、速度、位置与亚像素余量；玩家 dash-jump 与云 rider launch 都按源顺序执行。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [cloud_super_and_hyper_stack_the_cloud_lift_with_source_dash_jump_speeds / cloud_runtime_keeps_split_simulation_composable]),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [entity-4.6.1-cloud-hyper / entity-4.6.1-cloud-super],
    note: [两条真实 Playground 轨迹各 71 个状态；position 最大误差均为 0，speed 最大误差分别为 0.000961 与 0.000481，state、facing、dashes、stamina、grounded、ducking、death 全部一致。主录制使用 Cloud Hyper，Cloud Super 作为同机制的独立补充证明。],
  ),
  candidate-e2e: none,
)
