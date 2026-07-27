#import "../../template.typ": tech, evidence

#tech(
  id: "4.10.1",
  title-zh: "Dream Double-Jump",
  title-en: "Dream Double-Jump",
  status: "implemented",
  description-zh: [缓冲的 Dream Jump 不消耗出口土狼计时，因此离开梦块后还能再跳一次，额外获得高度和水平加速。],
  description-en: [A buffered dream jump can leave coyote time intact, permitting a second exit jump for more height and another horizontal boost.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Monocle/VirtualButton.cs],
    symbol: [Player.DreamDashUpdate / Player.DreamDashEnd / Player.NormalUpdate / VirtualButton.Update],
    note: [水平 DreamDash 出口先消费当帧 Jump 并调用普通 Jump，DreamDashEnd 随后仍恢复 0.1 秒 jump grace。`Celeste.Freeze(0.05)` 期间 Scene.Update 暂停，但 MInput/VirtualButton 继续推进并保存新的 Jump buffer；解冻后的首个 NormalUpdate 因 grace 尚在而执行第二次 Jump。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [advance_virtual_buttons / step / interact],
    note: [输入缓冲在全局 freeze 的提前返回之前推进；Dream Jump 按原版顺序留下横向出口 grace，解冻后 NormalUpdate 可消费 freeze 中记录的第二次跳跃。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [jump_buffer_survives_dream_exit_freeze_for_the_second_jump],
  ),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [entity-4.10.1-dream-double-jump],
    note: [原版第二章 `lvl_1` 场景共 37 个状态；position 最大误差 0、speed 最大误差 0.000001，state、facing、dashes、stamina、grounded、ducking、death 全部一致。第二次跳跃帧水平速度为约 315.667，确认额外的 +40 水平增速实际命中。],
  ),
  candidate-e2e: none,
)
