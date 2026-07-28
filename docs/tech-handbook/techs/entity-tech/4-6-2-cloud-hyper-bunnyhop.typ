#import "../../template.typ": tech, evidence

#tech(
  id: "4.6.2",
  title-zh: "云朵 Hyper 兔跳",
  title-en: "Cloud Hyper Bunnyhop",
  status: "unimplemented",
  description-zh: [在白云边缘做极短反向 Demo Hyper，并在云朵最高点高度重新落地后兔跳，以同时保留 Hyper 水平速度和云顶高度。Rust 已完成单条 runtime 回归与独立候选；真实 Everest 九字段对照前保持未实现。],
  description-en: [A short reverse demohyper at the cloud edge, followed by a landing and bunnyhop at the cloud-apex height, preserves hyper speed at maximum height. Rust now has one complete runtime regression and an independent candidate; the verdict stays unimplemented pending the real nine-field Everest comparison.],
  source-evidence: evidence(
    path: [Source/Cloud.cs; Source/Player/Player.cs],
    symbol: [Cloud.Update; Player.NormalUpdate; Player.Jump],
    snippet: raw(block: true, lang: "cs", "if (speed >= -100f && playerRider2?.Speed.Y >= 0f)\n    playerRider2.Speed.Y = -200f;\n...\nif (jumpGraceTimer > 0f && Input.Jump.Pressed)\n    Jump();"),
    note: [Cloud 反弹速度回升到 -100px/s 时把仍骑乘且非上升的玩家设为 -200；完整技巧则先用短 Hyper 离开，在云顶再次落地后由 jump grace／Jump buffer 调用普通 Jump。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [advance_clouds; normal_update], note: [同一 runtime trace 保存云的 phase/position/remainder；反向 Demo Hyper 离开白云后落到与云顶最低 Y 完全相同的相邻 8px 网格平台，再由普通 Jump 延续高速。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [cloud_hyper_completes_an_apex_bunnyhop_in_one_runtime_trace], note: [完整 45 帧回归断言反向 Demo Dash、325 Hyper、云顶高度 grounded 与下一帧 `(>250,-175.00047)` 兔跳，并验证 32 帧处分段重放一致。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.6.2-cloud-hyper-bunnyhop.ts; scripts/e2e-real/scenarios/common-parts.ts], symbol: [entity-4.6.2-cloud-hyper-bunnyhop; TECH_ENTITY_4_6_2_CLOUD_HYPER_BUNNYHOP], note: [独立 MapPart 使用真实 Cloud 与对齐到其 18px 顶点位移的 8px 网格平台；候选按 Rust 回归帧序执行反向 Demo Hyper 与云顶高度兔跳，现等真实 Everest 九字段确认。]),
)
