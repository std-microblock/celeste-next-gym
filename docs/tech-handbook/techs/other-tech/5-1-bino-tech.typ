#import "../../template.typ": tech, evidence

#tech(
  id: "5.1",
  title-zh: "望远镜技巧总类",
  title-en: "Bino Tech",
  status: "implemented",
  description-zh: [望远镜先让玩家进入自动走位，再进入观察状态；两阶段的状态、镜头和交互标志可被切换与房间边界打断。],
  description-en: [Binoculars first force Dummy movement to their center, then run a separate camera-control coroutine until an exit input or the interaction flag ends it. Rust models that lifecycle, including the HUD-hide exit boundary, and the real semantic trace passes.],
  source-evidence: evidence(
    path: [Celeste/Lookout.cs; Source/Player/Player.cs],
    symbol: [Lookout.Interact; Lookout.LookRoutine; Player.DummyWalkToExact],
    snippet: raw(block: true, lang: "cs", "player.StateMachine.State = StDummy;\nyield return player.DummyWalkToExact((int) X);\n...\nwhile (!Input.MenuCancel.Pressed && !Input.Dash.Pressed && interacting) {\n    Vector2 value = Input.Aim.Value;\n    level.Camera.Position = cam;\n    yield return null;\n}\ninteracting = false;\nplayer.StateMachine.State = StNormal;"),
    note: [Lookout 的交互并非单一 Player 状态：先让 `DummyWalkToExact` 完成自动走位，再由实体协程直接读取 Aim、更新 Camera，最后独立清除 `interacting` 并恢复 Normal。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [LookoutSnapshot; try_begin_lookout; prepare_lookout_player; advance_lookouts], note: [快照独立保存 interaction phase、HUD easer、camStart/camSpeed 与节点游标；Talk 矩形进入 DummyWalk，0.2 秒等待、3/s HUD ease、Aim 镜头和退出恢复按实体协程推进。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [lookout_talk_runs_dummy_wait_hud_camera_and_exit_lifecycle], note: [回归逐段验证 Talk、Dummy、HUD/camera 与 jump exit，不用单一 Player 状态冒充实体生命周期。]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.1-bino-tech.ts; scripts/e2e-real/scenarios/lookout-parts.ts; mods/CelesteGymCollector/Source/CelesteGymCollectorModule.cs; crates/celeste-physics/src/sim.rs; .tmp/e2e-runs/2026-07-28T20-07-47.845Z-90116-7a9e9654-6802-4b01-b566-badc8680f771/manifest.json], symbol: [other-5.1-bino-tech; TECH_OTHER_5_1_BINO_TECH; InstallScriptedButtons; advance_lookout_exit], note: [候选 SHA `1507181f921118c3691045fd18df28a1e25e870a` 的 2026-07-28 隔离真实 Everest run 在物理 `vendor/celeste-game` 上执行；per-run manifest 记录隔离 save/tmp、动态端口 61269/61271、nonce `5e29d8b0-4eda-43b8-ab91-d993f70109d4` 与本次 spawned Celeste PID `110980` 的精确握手，以及受控 cleanup。151 个状态的 position、speed、state、facing、dashes、stamina、grounded、ducking、death 全部逐帧匹配，position/speed 最大误差均为 0。f0 Talk，f1 开始 interaction，f2 为 Dummy；相机水平位移最大 186.0001px；f129 仍为 Dummy/interacting，f130 恢复 Normal/non-interacting。]),
  candidate-e2e: none,
)
