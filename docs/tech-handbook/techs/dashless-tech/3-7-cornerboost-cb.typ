#import "../../template.typ": tech, evidence

#tech(
  id: "3.7",
  title-zh: "Cornerboost 墙角加速",
  title-en: "Cornerboost (cb)",
  status: "unimplemented",
  description-zh: [撞墙时保存的 retained speed 会在 5 帧内脱离阻挡后返还；在墙角攀跳可同时获得跳跃加速并取回原速度。],
  description-en: [Wall collision stores retained speed for five frames; climb-jumping past the corner refunds it while adding jump acceleration.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.NormalUpdate; Player.ClimbJump; Player.Jump; Player.OnCollideH; Player.Update],
    snippet: raw(block: true, lang: "cs", "if (Facing == Facings.Right && Input.Grab.Check && Stamina > 0)\n    ClimbJump();\n...\nSpeed.X += JumpHBoost * moveX;\n...\nif (wallSpeedRetentionTimer <= 0) {\n    wallSpeedRetained = Speed.X;\n    wallSpeedRetentionTimer = WallSpeedRetentionTime;\n}\nSpeed.X = 0;\n...\nelse if (!CollideCheck<Solid>(Position + Vector2.UnitX * Math.Sign(wallSpeedRetained))) {\n    Speed.X = wallSpeedRetained;\n    wallSpeedRetentionTimer = 0;\n}"),
    note: [NormalUpdate 可在 3px 墙跳探针内直接执行 ClimbJump；其 40 水平加速若在同帧撞角，会被 OnCollideH 存入 0.06 秒 retained-speed 窗口，清角后的 Update 再返还。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [normal_update; climb_jump; update_wall_speed_retention; move_exact],
    note: [模拟器按源码在 Normal 状态选择攀跳分支，碰撞时保存加速后的水平速度，并在一像素前方不再受阻时于状态回调前返还。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [cornerboost_climb_jump_stores_jump_boost_before_clearing_wall_top],
    note: [独立场景以 90 空速攀跳墙角，同帧保存 130，保持 Normal 与 82.5 体力；清角后计时器归零并返还高速。],
  ),
  e2e-evidence: none,
  candidate-e2e: "cornerboost",
)
