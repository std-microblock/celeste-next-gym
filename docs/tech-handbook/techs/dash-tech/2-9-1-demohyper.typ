#import "../../template.typ": tech, evidence

#tech(
  id: "2.9.1",
  title-zh: "Demo Hyper",
  title-en: "Demohyper",
  status: "implemented",
  description-zh: [水平 Demo 中角色已经蹲伏，因此跳出时会按 Hyper 而非 Super 处理，并保留更快的水平冲刺分量。],
  description-en: [Jumping from a horizontal demodash produces a hyper because the player is crouched, while retaining the faster horizontal dash component.],
  source-evidence: evidence(path: [Everest Celeste.Mod.mm/Patches/Player.cs; Source/Player/Player.cs], symbol: [CrouchDash patch; Player.SuperJump], note: [Everest crouch dash 保持蹲伏碰撞箱，随后复用原版 SuperJump 的 1.25/0.5 蹲伏倍率。]),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [begin_dash; dash_update; super_jump]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [demohyper_uses_crouched_super_launch_from_horizontal_demo]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [demohyper], note: [Everest demoDashed 进入后跳出速度 325/-52.5；九类核心字段最大误差 0。]),
  candidate-e2e: none,
)
