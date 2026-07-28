#import "../../template.typ": tech, evidence

#tech(
  id: "2.4",
  title-zh: "Wavedash 波冲",
  title-en: "Wavedash",
  status: "implemented",
  description-zh: [从空中向下斜冲刺落地并及时起跳，得到与 Hyper 相同的速度和高度，但对起步地面长度要求更低。],
  description-en: [Starting a down-diagonal dash in the air, landing, and jumping in time reproduces hyper speed and height with less runway.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.OnCollideV; Player.DashUpdate; Player.SuperJump],
    snippet: raw(block: true, lang: "cs", "if (DashDir.X != 0 && DashDir.Y > 0 && Speed.Y > 0) {\n    DashDir.Y = 0;\n    Speed.Y = 0;\n    Speed.X *= DodgeSlideSpeedMult;\n    Ducking = true;\n}\n...\nif (DashDir.Y == 0 && Input.Jump.Pressed && jumpGraceTimer > 0)\n    SuperJump();"),
    note: [空中下斜冲刺落地时转成 1.2 倍水平蹲伏滑行；若落地碰撞发生在 DashUpdate 的按跳检查之后，VirtualButton 会把 `Input.Jump.Pressed` 保留到下一帧，再由水平 DashUpdate 跳成 Hyper。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [step; move_exact; dash_update; super_jump], note: [每帧先把 VirtualButton 的剩余缓冲映射回 `jump_pressed`，因此落地碰撞把斜向 DashDir 转为水平后，下一 DashUpdate 仍能消费同一次按跳。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [wavedash_landing_converts_down_diagonal_dash_to_hyper; wavedash_buffers_jump_at_the_fourteen_pixel_minimum_height; thirteen_pixel_wavedash_control_jumps_before_dash_refill]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/area-1/wavedash.ts], symbol: [wavedash; verifyWavedashMinimumHeight], note: [14 像素起始高度的真实场景共 19 个状态：state 9 与地面齐平但仍为斜向 Dash，state 10 的落地碰撞转为 203.64676 水平蹲伏滑行并保留按跳，state 11 回填 Dash 后以 325/-52.5 起跳；13 像素本地控制在回填前起跳。position/speed 最大误差均为 0，其余七类核心字段逐帧一致。]),
  candidate-e2e: none,
)
