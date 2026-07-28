#import "../../template.typ": tech, evidence

#tech(
  id: "4.6.2",
  title-zh: "云朵 Hyper 兔跳",
  title-en: "Cloud Hyper Bunnyhop",
  status: "unimplemented",
  description-zh: [在白云边缘做极短 Hyper（常用反向 Demo Hyper），并在云朵最高点兔跳，以同时获得高速和最大高度。],
  description-en: [A very short edge hyper, often a reverse demohyper, followed by a bunnyhop at the cloud apex combines speed with maximum cloud height.],
  source-evidence: evidence(
    path: [Source/Cloud.cs / Source/Player/Player.cs],
    symbol: [Cloud.Update / Player.SuperJump / Player.NormalUpdate],
    snippet: raw(block: true, lang: "cs", "if (speed >= -100f && playerRider2?.Speed.Y >= 0f)\n    playerRider2.Speed.Y = -200f;\n...\nif (jumpGraceTimer > 0f && Input.Jump.Pressed)\n    Jump();"),
    note: [完整技巧要求短 Hyper 先保留水平速度，随后在云顶重新落地并由 jump grace／Jump buffer 触发兔跳；仅命中 Hyper 不足以证明该链。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [advance_clouds / super_jump / normal_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [cloud_hyper_speed_composes_with_an_apex_bunnyhop_snapshot]),
  e2e-evidence: none,
  candidate-e2e: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [entity-4.6.2-cloud-hyper-bunnyhop],
    note: [真实候选轨迹已在第 29 状态命中 Hyper；到第 62 状态仍未再次落地，语义断言得到 `hyper=29, bunnyhop=undefined`，所以没有把单独 Hyper 冒充为云顶兔跳，保持未实现。],
  ),
)
