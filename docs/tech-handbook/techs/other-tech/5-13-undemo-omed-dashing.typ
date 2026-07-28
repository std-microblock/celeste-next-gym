#import "../../template.typ": tech, evidence

#tech(
  id: "5.13",
  title-zh: "Undemo 反蹲冲",
  title-en: "Undemo (omed) Dashing",
  status: "implemented",
  description-zh: [DashBegin 先把 DashDir 清零，并按启动输入决定是否使用蹲伏碰撞箱；DashCoroutine yield 一帧后才从 lastAim 读取实际方向。Rust 已复现这个重定向窗口并加入独立候选，但在真实 Everest 九字段对照完成前保持未实现。],
  description-en: [DashBegin clears DashDir and selects the duck collider from launch input; DashCoroutine reads lastAim only after yielding one frame. Rust now reproduces this redirection window and has an independent candidate, but the verdict stays unimplemented pending the real nine-field Everest comparison.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.DashBegin; Player.DashCoroutine],
    snippet: raw(block: true, lang: "cs", "private void DashBegin() {\n    Speed = Vector2.Zero;\n    DashDir = Vector2.Zero;\n}\nprivate IEnumerator DashCoroutine() {\n    yield return null;\n    var dir = lastAim;\n    Speed = dir * DashSpeed;\n    DashDir = dir;\n}"),
    note: [碰撞箱判定发生在协程读取实际 aim 之前；以水平输入启动并在全局 freeze 期间改为向下，可得到向下 DashDir 与站立碰撞箱。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [begin_dash; dash_update], note: [begin_dash 按源码清空 dash_dir，但仍用启动帧的普通/蹲冲输入决定 ducking；dash_update 在对应协程恢复帧从逐帧更新的 last_aim 写入方向和 240px/s 速度。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [undemo_redirects_after_dash_begin_without_changing_the_standing_collider], note: [精确回归逐帧断言启动后 DashDir 为零、demo_dashed 与 ducking 为 false，并在冻结结束后得到 DashDir=(0,1)、Speed=(0,240) 且仍未蹲伏。]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.13-undemo-omed-dashing.ts; scripts/e2e-real/scenarios/common-parts.ts], symbol: [other-5.13-undemo-omed-dashing; TECH_OTHER_5_13_UNDEMO_DASHING], note: [独立 MapPart 以水平普通冲启动，随后在 freeze 窗口持续向下；真实 Everest 语义门确认 DashDir=(0,1)、Speed=(0,240) 且 ducking=false。11 个状态的 position 与 speed 最大误差均为 0，其余七类字段逐帧一致。]),
  candidate-e2e: none,
)
