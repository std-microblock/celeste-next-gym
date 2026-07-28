#import "../../template.typ": tech, evidence

#tech(
  id: "5.11",
  title-zh: "Spinner 长时冻结",
  title-en: "Spinner Freeze",
  status: "unimplemented",
  description-zh: [Level.TimeActive 使用单精度累计；数值到达 2^19 秒时 ULP 为 1/16 秒，`+1/60` 不再改变它。由于较早阶段的量化会让时钟快进，实际墙钟约 116–118 小时。offset 分组随后会永久命中或错过。Rust 已复现，但真实真机长时证据尚缺，故保持未实现。],
  description-en: [At TimeActive = 2^19 seconds, the f32 ULP is 1/16 second and adding 1/60 no longer changes the value. Earlier quantization makes wall time roughly 116–118 hours. Offset groups then permanently fire or miss. Rust reproduces this, but real long-running Everest evidence is still absent.],
  source-evidence: evidence(
    path: [Monocle/Scene.cs; Celeste/CrystalStaticSpinner.cs],
    symbol: [Scene.TimeActive; Scene.OnInterval; CrystalStaticSpinner.Update],
    snippet: raw(block: true, lang: "cs", "public float TimeActive;\nTimeActive += Engine.DeltaTime;\n...\nreturn Math.Floor((TimeActive - offset - Engine.DeltaTime) / interval)\n     < Math.Floor((TimeActive - offset) / interval);"),
    note: [所有运算先以 float 完成再交给 Math.Floor；高数量级的 ULP 同时影响累加与 `TimeActive-offset-DeltaTime`，因此不同 offset 固定落入不同 interval 结果。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/types.rs], symbol: [scene_time_active; scene_on_interval; advance_spinners], note: [快照字段和全部 interval 算术保持 `f32`；每个 Spinner 持久化自己的 offset、Visible 与 Collidable。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [float32_scene_clock_freezes_spinner_interval_groups], note: [回归在 `524288f32` 断言 `TimeActive + 1/60 == TimeActive`，并扫描 offset 证明部分组恒命中、部分组恒错过。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.11-spinner-freeze.ts; scripts/e2e-real/scenarios/cassette-spinner-parts.ts], symbol: [other-5.11-spinner-freeze; TECH_OTHER_5_11_SPINNER_FREEZE], note: [独立 Spinner baseline 的首次真实运行同样在 frame 1 得到 Everest dead=true、Rust dead=false，确认缺口来自 fresh invisible Spinner 的构造期 Collidable；runtime 与回归已按源码修正，尚待一次 baseline 复核。该短场景不快进或注入 TimeActive，约 116–118 小时的真实 freeze semantic 仍完全未证明，verdict 保持 unimplemented。]),
)
