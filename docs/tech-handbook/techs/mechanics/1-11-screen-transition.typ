#import "../../template.typ": tech, evidence

#tech(
  id: "1.11",
  title-zh: "房间切换",
  title-en: "Screen Transition",
  status: "unimplemented",
  description-zh: [跨房间会恢复冲刺和体力；垂直切换还会调整纵向速度，帮助玩家穿过单向平台并避免立即掉回原房间。],
  description-en: [Room transitions refill dashes and stamina, while vertical transitions also adjust vertical speed to carry the player cleanly across the boundary.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste.Level.EnforceBounds; Celeste.Level.TransitionRoutine],
    symbol: [Player.BeforeUpTransition; Player.BeforeDownTransition; Player.TransitionTo; Player.OnTransition],
    snippet: raw(block: true, lang: "cs", "// up\nSpeed.X = 0;\nvarJumpSpeed = Speed.Y = JumpSpeed; // -105\nAutoJump = true;\n// down\nSpeed.Y = Math.Max(0, Speed.Y);\n// transition loop\nMoveTowardsX(target.X, 60f * Engine.DeltaTime);\nMoveTowardsY(target.Y, 60f * Engine.DeltaTime);\n// completion\nRefillDash(); RefillStamina();"),
    note: [Level.EnforceBounds 先按邻接房间判断方向并执行 Before*Transition；TransitionRoutine 以 0.65 秒相机时长和每轴 60 px/s 将玩家送入下一房，完成时 OnTransition 回填冲刺、体力并重置墙滑、土狼与强制移动计时。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [Map.transition_rooms; begin_transition; update_transition; enforce_level_bounds]),
  test-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/sim.rs], symbol: [selected_room_retains_adjacent_transition_bounds; upward_screen_transition_applies_source_launch_and_completion_refills; downward_screen_transition_clamps_upward_speed_before_transfer]),
  e2e-evidence: none,
  candidate-e2e: "mechanics-screen-transition-up",
)
