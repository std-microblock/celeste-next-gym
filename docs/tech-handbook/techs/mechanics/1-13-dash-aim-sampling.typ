#import "../../template.typ": tech, evidence

#tech(
  id: "1.13",
  title-zh: "冲刺方向延迟采样",
  title-en: "Dash Aim Sampling",
  status: "implemented",
  description-zh: [按下冲刺只会进入 Dash 并触发冻结；方向不会在按键帧锁死。`DashCoroutine` 越过首个 yield、冻结结束后才读取最新瞄准方向，因此第 2 帧按冲刺并从第 3 帧开始按左会向左冲刺。],
  description-en: [Pressing dash enters Dash and starts the freeze without locking a direction. DashCoroutine reads the latest aim only after its initial yield and the freeze, so a dash pressed on frame 2 followed by left from frame 3 launches left.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.DashBegin; Player.DashCoroutine; Player.Update],
    snippet: raw(block: true, lang: "cs", "Speed = Vector2.Zero;\nDashDir = Vector2.Zero;\n...\nyield return null;\nvar dir = lastAim;\nvar newSpeed = dir * DashSpeed;\nSpeed = newSpeed;\nDashDir = dir;"),
    note: [Player.Update 在 `base.Update()` 推进状态机与协程之前刷新 `lastAim`。DashBegin 明确清空速度和方向；协程先 yield 一次，真实的 240 速度与 DashDir 因而使用恢复更新帧的最新方向，而不是冲刺按键帧的方向。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [step; begin_dash; dash_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [dash_direction_is_sampled_when_coroutine_resumes_after_freeze]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/mechanics-dash-aim-sampling.ts], symbol: [mechanics-dash-aim-sampling], note: [真实 Everest 与 Rust 均在第 2 状态帧进入 Dash、保持零速度与零 DashDir；冻结结束后的第 6 状态帧读取左输入，得到 DashDir=(-1,0)、speed=(-240,0) 并面向左。17 个状态的九类核心字段逐帧一致，最大 position／speed 误差均为 0。]),
  candidate-e2e: none,
)
