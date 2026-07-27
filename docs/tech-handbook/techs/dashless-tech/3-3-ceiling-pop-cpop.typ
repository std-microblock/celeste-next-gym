#import "../../template.typ": tech, evidence

#tech(
  id: "3.3",
  title-zh: "Ceiling Pop 天花板弹出",
  title-en: "Ceiling Pop (cpop)",
  status: "unimplemented",
  description-zh: [在墙底端的精确像素和子像素位置抓墙下滑，并利用仍可攀跳的一帧，从天花板下方获得向前位移。],
  description-en: [Precise grab and subpixel positioning at the bottom of a wall leaves one climb-jump frame that can pop the player forward beneath a ceiling.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.ClimbUpdate; Player.ClimbJump; Player.Update],
    snippet: raw(block: true, lang: "cs", "//Wall Jump\nif (Input.Jump.Pressed && (!Ducking || CanUnDuck)) {\n    if (moveX == -(int)Facing) WallJump(-(int)Facing);\n    else ClimbJump();\n    return StNormal;\n}\n...\n//No wall to hold\nif (!CollideCheck<Solid>(Position + Vector2.UnitX * (int)Facing))\n    return StNormal;\n...\nMoveH(Speed.X * Engine.DeltaTime, onCollideH);\nMoveV(Speed.Y * Engine.DeltaTime, onCollideV);"),
    note: [ClimbUpdate 在松抓与失去 1 像素墙面接触之前先处理跳跃。前一帧垂直移动刚滑过悬墙底部后，状态仍是 Climb；下一帧可先 ClimbJump。Player.Update 又先 MoveH 后 MoveV，因此先获得向前位移，再由天花板阻止上升，形成 ceiling pop。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [climb_update; step; move_axis],
    note: [模拟器按源码把 climb jump/dash 放在 let-go 与 lost-wall 检查之前，并保持水平移动先于垂直移动。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [ceiling_pop_climb_jumps_before_the_lost_wall_check],
    note: [标准悬墙下抓墙向下移动；第 18 输入帧进入“Climb 状态但已无 1px 墙接触”的唯一窗口。该帧攀跳先水平前移，再垂直撞顶归零，并核对状态、体力、蹲伏、接地与死亡。],
  ),
  e2e-evidence: none,
  candidate-e2e: "ceiling-pop",
)
