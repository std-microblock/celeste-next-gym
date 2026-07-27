#import "../../template.typ": tech, evidence

#tech(
  id: "1.11",
  title-zh: "房间切换",
  title-en: "Screen Transition",
  status: "implemented",
  description-zh: [跨房间会恢复冲刺和体力；向上切换先施加 0/-105，再以每轴 60 px/s 移到目标房间 Bottom - 5，并在 0.65 秒相机移动后的下一次协程恢复帧完成切房。],
  description-en: [Room transitions refill dashes and stamina. An upward transition first applies 0/-105, moves each axis at 60 px/s to the target room's Bottom - 5 point, and completes on the coroutine resume after the 0.65-second camera move.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste.Level.EnforceBounds; Celeste.Level.TransitionRoutine],
    symbol: [Player.BeforeUpTransition; Player.BeforeDownTransition; Player.TransitionTo; Player.OnTransition],
    snippet: raw(block: true, lang: "cs", "// up\nSpeed.X = 0;\nvarJumpSpeed = Speed.Y = JumpSpeed; // -105\nAutoJump = true;\n// down\nSpeed.Y = Math.Max(0, Speed.Y);\n// transition loop\nMoveTowardsX(target.X, 60f * Engine.DeltaTime);\nMoveTowardsY(target.Y, 60f * Engine.DeltaTime);\n// completion\nRefillDash(); RefillStamina();"),
    note: [Level.EnforceBounds 先按邻接房间判断方向并执行 Before*Transition；TransitionRoutine 以 0.65 秒相机时长和每轴 60 px/s 将玩家送入下一房，完成时 OnTransition 回填冲刺、体力并重置墙滑、土狼与强制移动计时。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [Map.transition_rooms; begin_transition; update_transition; enforce_level_bounds]),
  test-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/sim.rs], symbol: [selected_room_retains_adjacent_transition_bounds; upward_screen_transition_applies_source_launch_and_completion_refills; downward_screen_transition_clamps_upward_speed_before_transfer]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [mechanics-screen-transition-up; verifyUpwardScreenTransition], note: [真实 43 帧向上切房场景确认 BeforeUpTransition、Bottom - 5 目标点、40 帧完成间隔和延迟资源恢复；九类核心字段最大位置误差 0.000011、速度误差 0。]),
  candidate-e2e: none,
)
