#import "../../template.typ": tech, evidence

#tech(
  id: "5.1",
  title-zh: "望远镜技巧总类",
  title-en: "Bino Tech",
  status: "unimplemented",
  description-zh: [望远镜先让玩家进入自动走位，再进入观察状态；两阶段的状态、镜头和交互标志可被切换与房间边界打断。],
  description-en: [Binoculars first force Dummy movement to their center, then run a separate camera-control coroutine until an exit input or the interaction flag ends it. Rust has no Lookout, TalkComponent, camera/HUD coroutine, or interaction lifetime, so this remains a mechanism audit.],
  source-evidence: evidence(
    path: [Celeste/Lookout.cs; Source/Player/Player.cs],
    symbol: [Lookout.Interact; Lookout.LookRoutine; Player.DummyWalkToExact],
    snippet: raw(block: true, lang: "cs", "player.StateMachine.State = StDummy;\nyield return player.DummyWalkToExact((int) X);\n...\nwhile (!Input.MenuCancel.Pressed && !Input.Dash.Pressed && interacting) {\n    Vector2 value = Input.Aim.Value;\n    level.Camera.Position = cam;\n    yield return null;\n}\ninteracting = false;\nplayer.StateMachine.State = StNormal;"),
    note: [Lookout 的交互并非单一 Player 状态：先让 `DummyWalkToExact` 完成自动走位，再由实体协程直接读取 Aim、更新 Camera，最后独立清除 `interacting` 并恢复 Normal。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/sim.rs], symbol: [EntityKind; step], note: [当前实体枚举没有 Lookout，输入模型没有 Talk/MenuConfirm/MenuCancel，快照也没有 Camera、HUD 或交互协程状态；只复用 Dummy/Normal 无法保持源码中的两阶段生命周期。]),
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
