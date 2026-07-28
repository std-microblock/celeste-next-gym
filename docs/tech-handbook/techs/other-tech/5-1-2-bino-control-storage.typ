#import "../../template.typ": tech, evidence

#tech(
  id: "5.1.2",
  title-zh: "望远镜控制存储",
  title-en: "Bino Control Storage",
  status: "unimplemented",
  description-zh: [通过打断 Dummy／望远镜状态转换，可让玩家在镜头仍受望远镜控制时恢复移动。],
  description-en: [An interruption can restore the player to Normal without clearing Lookout.interacting, so player movement and binocular camera control run together. Rust represents this split state, but a real interruption trace is still required.],
  source-evidence: evidence(
    path: [Celeste/Lookout.cs; Source/Player/Player.cs],
    symbol: [Lookout.LookRoutine; Player.DummyWalkToExact],
    snippet: raw(block: true, lang: "cs", "player.StateMachine.State = StDummy;\nyield return player.DummyWalkToExact((int) X);\n...\nwhile (!Input.MenuCancel.Pressed && !Input.Dash.Pressed && interacting) {\n    Vector2 value = Input.Aim.Value;\n    level.Camera.Position = cam;\n    yield return null;\n}"),
    note: [相机循环的继续条件检查输入与 Lookout 自身的 `interacting`，不检查 Player 是否仍为 Dummy；因此打断 Player 状态转换而不结束实体交互，会让 Normal 移动与 Aim 相机控制同时存在。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [LookoutSnapshot; update_camera; advance_lookouts], note: [Lookout interaction 不依附 Player state；phase 4 会锁住常规相机但不强制 Normal 回 Dummy，因此 Normal movement 与 Aim camera 可同帧并行。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [bino_control_storage_keeps_normal_player_and_camera_control_parallel], note: [回归从 Talk 起步，用原生 Booster 进入 Boost 打断 Dummy，随后验证 `Normal + interacting`、玩家横移与 Lookout 镜头同帧变化，并由 jump 清除 flag。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.1.2-bino-control-storage.ts; mods/CelesteGymCollector/Source/CelesteGymCollectorModule.cs; crates/celeste-physics/src/sim.rs], symbol: [other-5.1.2-bino-control-storage; InstallScriptedButtons; bino_control_storage_keeps_normal_player_and_camera_control_parallel], note: [2026-07-28 的 trace 在 frame 180 未清除 stored interaction。收集器现将 portable jump 同步到 LookRoutine 实际读取的 `Input.MenuCancel`，候选输入在 Booster 恢复 Normal 后的 frame 220 退出，并有本地存储/退出回归；保持 candidate，待真实 Everest 重跑。]),
)
