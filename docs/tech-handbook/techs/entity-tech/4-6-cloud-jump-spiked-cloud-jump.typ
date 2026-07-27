#import "../../template.typ": tech, evidence

#tech(
  id: "4.6",
  title-zh: "云朵跳／尖刺云跳",
  title-en: "Cloud Jump / Spiked Cloud Jump",
  status: "implemented",
  description-zh: [先压下云朵，再在它上升到最高点时起跳，才能获得最大纵向加速；时序正确时可避开云下尖刺。],
  description-en: [Depress a cloud, then jump as it reaches its highest rebound to gain maximum vertical speed and clear hazards beneath it.],
  source-evidence: evidence(
    path: [Source/Cloud.cs],
    symbol: [Cloud.Update],
    snippet: raw(block: true, lang: "cs", "if (playerRider != null && playerRider.Speed.Y >= 0f)\n    speed = 180f;\n...\nif (Y >= startY) speed -= 1200f * Engine.DeltaTime;\nelse {\n    speed += 1200f * Engine.DeltaTime;\n    if (speed >= -100f && playerRider2?.Speed.Y >= 0f)\n        playerRider2.Speed.Y = -200f;\n}\nfloat lift = speed < 0f ? -220f : speed;\nMoveV(speed * Engine.DeltaTime, lift);"),
    note: [骑乘者下压时云以 180 起步，随后以每秒 1200 反向加速；越过起点且回升速度达到 -100 时把仍站在上面的玩家设为 -200。云上升时对 rider 报告固定 -220 lift，保证回弹与移动平台携带顺序一致。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/map.rs / crates/celeste-physics/src/sim.rs / crates/celeste-physics/src/types.rs], symbol: [EntityKind.Cloud / CloudSnapshot / advance_clouds / move_cloud_v], note: [Cloud 的 phase、位置、速度和余数全部进入快照；Player frame 后推进云并按原版阈值发射 rider，普通与带下方尖刺的独立 MapPart 共用机制。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [cloud_depresses_then_launches_the_rider_at_the_source_threshold / spiked_cloud_jump_keeps_the_rider_clear_of_the_hazard_below / cloud_runtime_keeps_split_simulation_composable]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [entity-4.6-cloud-jump], note: [真实原版 Cloud 从 `(616,440)` 站立启动；状态帧 24 位于 `(616,427)` 并获得 `(0,-200)`，全程未死亡。71 个状态九类字段逐帧一致，position 与 speed 最大误差均为 0。]),
  candidate-e2e: none,
)
