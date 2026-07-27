#import "../../template.typ": tech, evidence

#tech(
  id: "3.7.3",
  title-zh: "双 Cornerboost",
  title-en: "Double Cornerboost (dcb)",
  status: "implemented",
  description-zh: [在水平速度低于约 144 且像素位置精确时，可连续两帧攀跳同一墙角，额外获得两次加速。],
  description-en: [With precise positioning below roughly 144 horizontal speed, two consecutive climb jumps can boost from the same corner.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.NormalUpdate; Player.WallJumpCheck; Player.ClimbJump; Player.Jump; Player.OnCollideH; Player.Update],
    snippet: raw(block: true, lang: "cs", "if (canUnduck && WallJumpCheck(1))\n    ClimbJump();\n...\nSpeed.X += JumpHBoost * moveX;\n...\nwallSpeedRetained = Speed.X;\nwallSpeedRetentionTimer = WallSpeedRetentionTime;"),
    note: [NormalUpdate 每帧都可在 3px 探针内执行 ClimbJump；第一次攀跳先增加 40 并向墙角移动，精确像素位置可让下一帧仍命中同一探针，第二次再增加 40，随后碰撞把两次加速后的速度存入 0.06 秒 retention。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [normal_update; climb_jump; wall_jump_check; move_axis_amount; update_wall_speed_retention],
    note: [模拟器逐帧保留源码的输入缓冲、3px 墙跳探针、JumpHBoost、整数像素移动余量和碰撞后速度保留顺序，不为双 Cornerboost 直接注入速度或 retention。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [double_cornerboost_uses_two_consecutive_climb_jumps_from_a_grounded_setup],
    note: [独立场景从地面零速起跳、接墙并攀到墙顶，再松抓调整一像素位置；连续两帧攀跳各扣 27.5 体力，第二次自然保存 90.83336，并在计时窗内清角返还。],
  ),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [double-cornerboost],
    note: [真实游戏共 91 个状态帧，从 120/152 地面零速起跳并自然攀墙；第 79 状态帧由松抓输入链进入 139/87 的同角窗口，第 80、81 状态帧连续攀跳后体力由 72.1212 降至 44.6212、17.1212，第二次自然保存 90.83336，第 85 状态帧清角返还 90。九类核心字段逐帧一致，最大位置误差 0、速度误差 0.000008。],
  ),
  candidate-e2e: none,
)
