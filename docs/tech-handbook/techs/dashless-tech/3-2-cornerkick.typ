#import "../../template.typ": tech, evidence

#tech(
  id: "3.2",
  title-zh: "Cornerkick 墙角踢",
  title-en: "Cornerkick",
  status: "implemented",
  description-zh: [从墙角正下方掠过时对角落执行墙跳，可获得少量高度；无方向输入时会形成 neutral 版本。],
  description-en: [A wallkick taken just under a corner gains height from an otherwise tiny wall contact, with a neutral form when no direction is held.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.NormalUpdate; Player.WallJumpCheck; Player.WallJump],
    snippet: raw(block: true, lang: "cs", "private const int WallJumpCheckDist = 3;\n...\nreturn ClimbBoundsCheck(dir) && CollideCheck<Solid>(Position + Vector2.UnitX * dir * WallJumpCheckDist);\n...\nif (canUnduck && WallJumpCheck(1))\n    WallJump(-1);"),
    note: [普通攀抓只探测 2 像素，而 WallJumpCheck 横向探测 3 像素。玩家 8×11 碰撞箱从墙角下方上升到最后一个重叠像素时，即使仍有 2 像素水平间隙也能命中角点并执行墙跳；无方向输入时不会设置强制离墙计时。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [WALL_JUMP_CHECK_DIST; climb_check; wall_jump_check; normal_update],
    note: [模拟器分别保留 2 像素 ClimbCheck 与 3 像素 WallJumpCheck，并在 NormalUpdate 的跳跃阶段按源码顺序探测两侧墙面。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [cornerkick_uses_the_three_pixel_probe_on_the_last_corner_pixel],
    note: [测试构造悬墙底角：1px 接触与 2px 攀抓探针均失败，3px 墙跳探针在最后一个垂直像素命中；同时覆盖方向墙跳、neutral 墙跳及再低 1px 后失败。],
  ),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [cornerkick],
    note: [真实游戏从墙角下方 242/90 起步，共 13 个状态帧；首个输入帧命中 3px 墙跳探针并得到 -130/-105。九类核心字段逐帧一致，最大位置误差 0、速度误差 0.000008。],
  ),
  candidate-e2e: none,
)
