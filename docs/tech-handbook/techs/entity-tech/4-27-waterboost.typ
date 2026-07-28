#import "../../template.typ": tech, evidence

#tech(
  id: "4.27",
  title-zh: "Waterboost 水面加速",
  title-en: "Waterboost",
  status: "implemented",
  description-zh: [在水面接触窗口内可连续多次起跳，每次增加约 40 水平速度；人类通常只能完成少量连续跳。],
  description-en: [Repeated jumps on the water surface each add about 40 horizontal speed, with TAS timing allowing many more than humans usually manage.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Source/Water.cs],
    symbol: [Player.NormalUpdate / Player.Jump / Water.TopSurface],
    snippet: raw(block: true, lang: "cs", "else if ((water = CollideFirst<Water>(Position + Vector2.UnitY * 2f)) != null)\n{\n    Jump();\n    water.TopSurface.DoRipple(Position, 1f);\n}\n...\nSpeed.X += 40f * (float)moveX;"),
    note: [NormalUpdate 在 grace 与两侧墙跳均失败后，以玩家下方 2 像素探针检查水面并调用普通 Jump。Jump 每次都按当前输入方向叠加固定 40 水平速度；只要短暂水面接触窗口仍成立，连续缓冲跳就能重复叠加。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs / crates/celeste-physics/src/map.rs], symbol: [normal_update / current_player_rect / Map.water_at]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [water_surface_jumps_can_stack_multiple_forty_speed_boosts]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.27-waterboost.ts], symbol: [entity-4.27-waterboost], note: [独立水池 MapPart 连续三帧跳跃，速度依次约为 50、100、135.66666，第三次 Y 为 -105；真实 Everest 共 25 个状态，九类字段全部逐帧一致，最大 position／speed 误差均为 0。]),
  candidate-e2e: none,
)
