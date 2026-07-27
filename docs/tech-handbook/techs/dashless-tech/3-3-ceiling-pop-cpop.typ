#import "../../template.typ": tech, evidence

#tech(
  id: "3.3",
  title-zh: "Ceiling Pop 天花板弹出",
  title-en: "Ceiling Pop (cpop)",
  status: "implemented",
  description-zh: [在墙底端的精确像素和子像素位置抓墙下滑，并利用仍可攀跳的一帧，从天花板下方获得向前位移。],
  description-en: [Precise grab and subpixel positioning at the bottom of a wall leaves one climb-jump frame that can pop the player forward beneath a ceiling.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.ClimbUpdate; Player.ClimbJump; Player.NormalBegin; Player.NormalUpdate; Player.Update],
    snippet: raw(block: true, lang: "cs", "private void NormalBegin() {\n    maxFall = MaxFall;\n}\n...\nif (Input.MoveY == 1 && Speed.Y >= mf)\n    maxFall = Calc.Approach(maxFall, fmf, FastMaxAccel * Engine.DeltaTime);\n...\n//Wall Jump\nif (Input.Jump.Pressed && (!Ducking || CanUnDuck)) {\n    if (moveX == -(int)Facing) WallJump(-(int)Facing);\n    else ClimbJump();\n    return StNormal;\n}\n...\n//No wall to hold\nif (!CollideCheck<Solid>(Position + Vector2.UnitX * (int)Facing))\n    return StNormal;\n...\nMoveH(Speed.X * Engine.DeltaTime, onCollideH);\nMoveV(Speed.Y * Engine.DeltaTime, onCollideV);"),
    note: [ClimbUpdate 在松抓与失去 1 像素墙面接触之前先处理跳跃。前一帧垂直移动刚滑过悬墙底部后，状态仍是 Climb；下一帧可走 ClimbJump 并进入 NormalBegin，把 maxFall 重置为 160。向下输入也只有在速度已达到普通下落上限时才启用快速下落。Player.Update 又先 MoveH 后 MoveV，因此先获得向前位移，再由天花板阻止上升，形成 ceiling pop。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [climb_update; enter_normal; normal_update; step; move_axis],
    note: [模拟器按源码把 climb jump/dash 放在 let-go 与 lost-wall 检查之前，所有 Climb 到 Normal 的出口执行 max_fall 重置，快速下落门槛要求速度已达到 160，并保持水平移动先于垂直移动。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [ceiling_pop_climb_jumps_before_the_lost_wall_check; fastfall_limit_stays_normal_until_downward_speed_reaches_160],
    note: [标准悬墙下抓墙向下移动；第 18 输入帧进入“Climb 状态但已无 1px 墙接触”的唯一窗口。该帧攀跳先水平前移，再垂直撞顶归零；测试还从 240 的旧 maxFall 起步，验证返回 Normal 后重置为 160，并核对九类核心状态字段。],
  ),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [ceiling-pop],
    note: [真实游戏共 31 个状态帧：第 18 状态帧仍是 Climb 且位于 244/91，第 18 输入帧攀跳后第 19 状态帧进入 Normal、前移到 245/91、速度为 40/0、体力为 82.5；末帧下落速度严格为 160。九类核心字段逐帧一致，最大位置误差 0、速度误差 0.000001。],
  ),
  candidate-e2e: none,
)
