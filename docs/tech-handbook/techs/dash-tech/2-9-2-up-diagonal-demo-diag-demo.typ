#import "../../template.typ": tech, evidence

#tech(
  id: "2.9.2",
  title-zh: "上斜 Demo",
  title-en: "Up Diagonal Demo (Diag Demo)",
  status: "implemented",
  description-zh: [使用蹲冲键向上斜冲刺可保持缩小的碰撞箱，常用于更宽容地越过天花板边缘并接 Cornerkick。],
  description-en: [An up-diagonal crouch dash keeps the reduced hitbox and is often used to clear a ceiling edge before a cornerkick.],
  source-evidence: evidence(path: [Everest Celeste.Mod.mm/Patches/Player.cs; Source/Player/Player.cs], symbol: [CrouchDash patch; Player.DashCoroutine], note: [Everest 允许蹲伏冲刺使用上斜瞄准，原版协程归一化方向并施加 240 冲刺速度。]),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [begin_dash; dash_update; current_player_rect]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [upward_diagonal_demo_keeps_crouched_dash_hitbox]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [up-diagonal-demo], note: [独立 Everest 场景保持 demoDashed/ducking，并达到 169.70563/-169.70563；最大误差 0。]),
  candidate-e2e: none,
)
