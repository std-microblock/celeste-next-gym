#import "../../template.typ": tech, evidence

#tech(
  id: "5.11",
  title-zh: "Spinner 长时冻结",
  title-en: "Spinner Freeze",
  status: "unimplemented",
  description-zh: [Level.TimeActive 使用单精度浮点累计；约 2^22 秒（约 116–118 小时）后其 ULP 接近/超过普通帧增量，时间可能不再逐帧变化。按 TimeActive+offset 分组的 Spinner interval 因此可永久只命中部分组。当前模拟器没有该 float32 场景时钟或 Spinner 分组。],
  description-en: [Level.TimeActive accumulates in single precision. Around 2^22 seconds (roughly 116–118 hours), its ULP approaches or exceeds a normal frame delta and the value may stop advancing each frame, causing TimeActive+offset spinner intervals to service only some groups indefinitely. The simulator has neither this float32 scene clock nor spinner grouping.],
  source-evidence: evidence(path: [Monocle/Scene.cs; Celeste/CrystalStaticSpinner.cs], symbol: [Scene.TimeActive; Scene.OnInterval; CrystalStaticSpinner.Update], note: [Scene 每帧以 float DeltaTime 累加 TimeActive；OnInterval 对当前/上一时间桶做比较，Spinner 再加各自 offset。高数量级下 float ULP 导致桶边界不再按 60 Hz 跨越，产生永久分组偏置。]),
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
