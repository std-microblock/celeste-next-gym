#import "../../template.typ": tech, evidence

#tech(
  id: "5.4",
  title-zh: "过场传送偏移",
  title-en: "Cutscene Warps",
  status: "product-excluded",
  description-zh: [某些跳过过场只重设水平坐标而保留垂直坐标，因此可让玩家出现在房间中异常高度。],
  description-en: [Some CutsceneEntity.OnEnd skip handlers replace only player X and restore Normal, leaving the pre-skip Y intact; triggering them from an unusual height produces a warp. Cutscene/menu control is outside the simulator scope requested by the user, so this is mechanism-only.],
  source-evidence: evidence(
    path: [Celeste/CS00_Granny.cs; Celeste/CutsceneEntity.cs],
    symbol: [CS00_Granny.OnEnd; CutsceneEntity.SkipCutscene],
    snippet: raw(block: true, lang: "cs", "public override void OnEnd(Level level) {\n    player.Position.X = endPlayerPosition.X;\n    player.Facing = Facings.Left;\n    player.StateMachine.State = StNormal;\n    level.Session.SetFlag(\"granny\", true);\n}"),
    note: [代表性的 Granny 跳过回调只覆盖 `Position.X`，没有写 Y，也没有用 `MoveTo` 做碰撞校正；因此跳过瞬间的垂直坐标被原样保留，再恢复 Normal。其他过场拥有各自的 OnEnd 落点规则，不能抽象成普通传送。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [PlayerState; step], note: [Rust 没有 CutsceneEntity、skip/menu 输入、NPC、session flags 或逐过场 OnEnd 回调。按产品决定不实现游戏外跳过控制，也不把直接改 X 的测试钩子伪装成技巧实现。]),
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
