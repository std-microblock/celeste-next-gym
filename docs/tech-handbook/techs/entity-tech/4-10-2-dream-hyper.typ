#import "../../template.typ": tech, evidence

#tech(
  id: "4.10.2",
  title-zh: "Dream Hyper",
  title-en: "Dream Hyper",
  status: "implemented",
  description-zh: [离开梦块后的土狼窗口内执行 Hyper 或 Demo Hyper，可获得约 325 水平速度；Demo 版本通常窗口更稳定。],
  description-en: [A hyper or demohyper during dream-exit coyote time produces about 325 horizontal speed, with the demo setup usually offering a steadier window.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.DreamDashEnd / Player.StartDash / Player.DashUpdate / Player.SuperJump],
    note: [横向 DreamDashEnd 恢复 0.1 秒 jump grace；该窗口内启动水平 dash 或 demodash 后，DashUpdate 在 `DashDir.Y == 0` 时消费 Jump，并由蹲伏 collider 选择 Hyper 的 SuperJump 分支，得到 325/-52.5。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [interact / begin_dash / dash_update / super_jump],
    note: [DreamDash 横向出口保留 grace；CrouchDash VirtualButton 可跨出口 freeze 缓冲，随后水平 demo dash 在 grace 内跳跃并使用 Hyper 的 1.25 水平与 0.5 垂直倍率。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [jump_buffer_survives_dream_exit_freeze_for_the_second_jump / demohyper_uses_crouched_super_launch_from_horizontal_demo],
  ),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [entity-4.10.2-dream-hyper],
    note: [原版第二章 `lvl_1` 中先横向离开 DreamBlock，再于 freeze 中缓冲水平 CrouchDash，并在出口 grace 内起跳。语义守卫确认 frame 25 达到 `(325,-52.5)`；39 个状态的 position 最大误差 0、speed 最大误差 0.000001，其余七类字段逐帧一致。],
  ),
  candidate-e2e: none,
)
