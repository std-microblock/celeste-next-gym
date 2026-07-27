#import "../../template.typ": tech, evidence

#tech(
  id: "4.10",
  title-zh: "Dream Jump 梦块跳",
  title-en: "Dream Jump",
  status: "implemented",
  description-zh: [梦块出口提供土狼时间；离开时起跳可获得跳跃高度和额外的 40 水平速度。],
  description-en: [Dream-block exits grant coyote time, allowing a jump that adds height and about 40 horizontal speed.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.DreamDashUpdate / Player.Jump / Player.DreamDashEnd],
    note: [DreamDash 的水平出口在退出帧读取 Jump buffer 并调用普通 Jump：纵速设为 -105，水平速度增加 `40 * moveX`。DreamDashEnd 再为横向出口恢复 0.1 秒 jump grace，并回填 dash 与 stamina。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [interact],
    note: [水平 DreamDash 出口在状态切换前执行普通跳跃常量与 +40 水平增速，然后按 DreamDashEnd 顺序恢复 grace、dash、stamina 并保留同帧第二次移动。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [dream_jump_runs_on_exit_and_restores_horizontal_exit_grace],
  ),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [entity-4.10-dream-jump],
    note: [原版第二章 `lvl_1` 场景共 33 个状态；position/speed 最大误差均为 0，九个比较字段全部一致。frame 16 离开 DreamDash 后速度为 `(280,-105)`、dash 恢复为 1，确认 Dream Jump 语义实际命中。],
  ),
  candidate-e2e: none,
)
