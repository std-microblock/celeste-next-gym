#import "../../template.typ": tech, evidence

#tech(
  id: "1.8",
  title-zh: "快速下落",
  title-en: "Fastfalling (Fastfall)",
  status: "implemented",
  description-zh: [空中按住下方向会逐步把最大下落速度从 160 提高到 240；松开后上限再平滑回落。],
  description-en: [Holding down in the air gradually raises terminal fall speed from 160 to 240, and releasing down eases it back toward 160.],
  source-evidence: evidence(path: [Source/Player/Player.cs], symbol: [Player.NormalUpdate], note: [向下输入把 maxFall 以 300/s 逼近 FastMaxFall=240，普通上限为 160。]),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [normal_update; FAST_MAX_FALL]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [fastfall_approaches_source_240_terminal_speed]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [fastfall], note: [真实游戏第 16 帧达到 240 下落速度；九类核心字段最大误差 0。]),
  candidate-e2e: none,
)
