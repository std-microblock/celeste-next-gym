#import "../../template.typ": tech, evidence

#tech(
  id: "4.14",
  title-zh: "Heart Ultra",
  title-en: "Heart Ultras",
  status: "unimplemented",
  description-zh: [在收集水晶心的同一帧向下斜冲刺，可利用心脏补冲和状态中断形成 Ultra；可先耗尽冲刺把输入变成缓冲。],
  description-en: [Down-diagonal dashing on the heart-collection frame combines the refill and interruption into an ultra, with dash exhaustion enabling a buffered setup.],
  source-evidence: evidence(
    path: [Celeste/HeartGem.cs (v1.4.0 decompile); Source/Player/Player.cs],
    symbol: [HeartGem.OnPlayer / HeartGem.CollectRoutine / Player.StartDash],
    note: [HeartGem 只在 DashAttacking 时开始 Collect；协程先跨一帧，再触发 0.2 秒 Freeze。Heart Ultra 依赖收集回调、补充冲刺与玩家 Dash 状态中断在同一更新序列中的精确先后。当前地图 schema 与 Rust 均没有 HeartGem、收集 freeze/refill 或该中断顺序，因此保留未实现。],
    snippet: raw(block: true, lang: "cs", "if (player.DashAttacking) {\n    Collect(player);\n    return;\n}\n...\nyield return null;\nCeleste.Freeze(.2f);\nyield return null;\nEngine.TimeRate = .5f;"),
  ),
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
