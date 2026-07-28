#import "../../template.typ": tech, evidence

#tech(
  id: "4.6.2",
  title-zh: "云朵 Hyper 兔跳",
  title-en: "Cloud Hyper Bunnyhop",
  status: "implemented",
  description-zh: [在白云边缘做极短反向 Demo Hyper，并在云朵最高点高度重新落地后兔跳，以同时保留 Hyper 水平速度和云顶高度。],
  description-en: [A short reverse demohyper at the cloud edge, followed by a landing and bunnyhop at the cloud-apex height, preserves hyper speed at maximum height.],
  source-evidence: evidence(
    path: [Source/Cloud.cs; Source/Player/Player.cs],
    symbol: [Cloud.Update; Player.NormalUpdate; Player.Jump],
    snippet: raw(block: true, lang: "cs", "if (speed >= -100f && playerRider2?.Speed.Y >= 0f)\n    playerRider2.Speed.Y = -200f;\n...\nif (jumpGraceTimer > 0f && Input.Jump.Pressed)\n    Jump();"),
    note: [Cloud 反弹速度回升到 -100px/s 时把仍骑乘且非上升的玩家设为 -200；完整技巧则先用短 Hyper 离开，在云顶再次落地后由 jump grace／Jump buffer 调用普通 Jump。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [advance_clouds; move_cloud_v], note: [Cloud 回程的 MoveTowardsY 以实际请求位移除以 DeltaTime 写入 LiftSpeed；JumpThru 上移则先检查目标位置相交且当前位置不相交。该顺序保留回程 lift，并使 Hyper 的 subpixel movementCounter 与原版一致。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [cloud_hyper_bunnyhop_fixture_leaves_the_platform_side_before_apex_landing], note: [45 帧 fixture 精确断言 f30 `(521,418)`、f35 `(547,415)` 及其 -0.5 vertical remainder，随后离开平台、落地并以大于 300 的水平速度兔跳。]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.6.2-cloud-hyper-bunnyhop.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs; crates/celeste-physics/examples/compare_real_trace.rs], symbol: [entity-4.6.2-cloud-hyper-bunnyhop; SnapshotCapture.Capture; compare_real_trace], note: [2026-07-29 在物理 vendor/celeste-game 上以隔离 save/tmp、动态端口、nonce 和 spawned child PID 握手运行 46 帧；position 与 speed 最大误差均为 0.000000，state、facing、dashes、stamina、grounded、ducking 与 death 亦逐帧一致。]),
)
