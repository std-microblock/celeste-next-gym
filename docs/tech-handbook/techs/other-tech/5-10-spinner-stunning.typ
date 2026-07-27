#import "../../template.typ": tech, evidence

#tech(
  id: "5.10",
  title-zh: "Spinner 暂停眩晕",
  title-en: "Spinner Stunning",
  status: "unimplemented",
  description-zh: [CrystalStaticSpinner 把昂贵的距离/碰撞启用检查错开到约每 0.05 秒一次；若在自己的检查帧暂停，Level/实体更新被跳过，下一次 interval 可能落到别组，延长无碰撞状态。它同时依赖 Spinner 实体和游戏外暂停，当前不实现。],
  description-en: [CrystalStaticSpinner staggers its proximity/collision activation check to roughly every 0.05 seconds. Pausing on its check frame skips Level/entity update so the next interval may service another group, extending the harmless state. It depends on both Spinner runtime and out-of-game pause, which are not implemented.],
  source-evidence: evidence(path: [Celeste/CrystalStaticSpinner.cs; Celeste/Level.cs; Monocle/Scene.cs], symbol: [CrystalStaticSpinner.Update; Scene.OnInterval; Level.Pause], note: [Spinner 以随机/分组 offset 调用 Scene.OnInterval(0.05)，仅在命中帧根据玩家距离切换 Collidable；暂停阻止该实体 Update，所以被跳过的 interval 不会补执行。]),
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
