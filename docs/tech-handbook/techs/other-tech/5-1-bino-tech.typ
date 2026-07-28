#import "../../template.typ": tech, evidence

#tech(
  id: "5.1",
  title-zh: "望远镜技巧总类",
  title-en: "Bino Tech",
  status: "unimplemented",
  description-zh: [望远镜先让玩家进入自动走位，再进入观察状态；两阶段的状态、镜头和交互标志可被切换与房间边界打断。],
  description-en: [Binoculars first force Dummy movement to their center, then run a separate camera-control coroutine until an exit input or the interaction flag ends it. Rust now models that lifecycle; the candidate stays unimplemented until its real semantic trace passes.],
  source-evidence: evidence(
    path: [Celeste/Lookout.cs; Source/Player/Player.cs],
    symbol: [Lookout.Interact; Lookout.LookRoutine; Player.DummyWalkToExact],
    snippet: raw(block: true, lang: "cs", "player.StateMachine.State = StDummy;\nyield return player.DummyWalkToExact((int) X);\n...\nwhile (!Input.MenuCancel.Pressed && !Input.Dash.Pressed && interacting) {\n    Vector2 value = Input.Aim.Value;\n    level.Camera.Position = cam;\n    yield return null;\n}\ninteracting = false;\nplayer.StateMachine.State = StNormal;"),
    note: [Lookout 的交互并非单一 Player 状态：先让 `DummyWalkToExact` 完成自动走位，再由实体协程直接读取 Aim、更新 Camera，最后独立清除 `interacting` 并恢复 Normal。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [LookoutSnapshot; try_begin_lookout; prepare_lookout_player; advance_lookouts], note: [快照独立保存 interaction phase、HUD easer、camStart/camSpeed 与节点游标；Talk 矩形进入 DummyWalk，0.2 秒等待、3/s HUD ease、Aim 镜头和退出恢复按实体协程推进。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [lookout_talk_runs_dummy_wait_hud_camera_and_exit_lifecycle], note: [回归逐段验证 Talk、Dummy、HUD/camera 与 jump exit，不用单一 Player 状态冒充实体生命周期。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.1-bino-tech.ts; crates/celeste-physics/src/sim.rs], symbol: [other-5.1-bino-tech; advance_lookout_exit; lookout_talk_runs_dummy_wait_hud_camera_and_exit_lifecycle], note: [2026-07-28 独立真实 Everest run `2026-07-28T10-29-54.404Z-87584-93b2f1b1-3abd-4d37-8eeb-64681df8c1f0` 已完成认证与清理；原 trace 的首个不一致在 frame 130：Rust=Dummy、Everest=Normal（position/speed 均为 0 误差）。候选现在按 LookRoutine 的同一 exit-input coroutine resume 开始 3/s HUD Hide，并以本地 frame 130 Normal 回归锁定；仍保持 candidate，待真实 Everest 重跑九字段比较。]),
)
