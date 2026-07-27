#import "../../template.typ": tech, evidence

#tech(
  id: "5.13",
  title-zh: "Undemo 反蹲冲",
  title-en: "Undemo (omed) Dashing",
  status: "unimplemented",
  description-zh: [DashBegin 首帧先把 DashDir 清零并按启动输入决定是否蹲伏；DashCoroutine yield 一帧后才从 lastAim 读取实际方向。先以非向下输入启动、冻结期间改为向下，即可得到向下 DashDir 而保留普通碰撞箱。当前 Rust 在 begin_dash 当帧直接缓存方向，尚未建模该重定向窗口。],
  description-en: [DashBegin first clears DashDir and decides crouching from launch input; DashCoroutine yields one frame before reading lastAim for the actual direction. Launching non-downward and redirecting downward during freeze produces a downward DashDir with the normal collider. Rust currently caches direction immediately in begin_dash and does not model this redirection window.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.DashBegin; Player.DashCoroutine],
    snippet: raw(block: true, lang: "cs", "private void DashBegin() {\n    Speed = Vector2.Zero;\n    DashDir = Vector2.Zero;\n    if (!onGround && Ducking && CanUnDuck) Ducking = false;\n}\nprivate IEnumerator DashCoroutine() {\n    yield return null;\n    Vector2 aim = lastAim;\n    Speed = DashDir = aim * DashSpeed;\n}"),
    note: [蹲伏判定发生在协程读取实际 aim 之前；中性/非向下启动不会设置 duck collider，而 yield 与 dash freeze 给输入重定向留下窗口。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [begin_dash; dash_update], note: [当前 begin_dash 直接把 input_aim 写入 dash_dir 并同时决定 ducking；要实现需拆分启动输入与下一次协程推进的 lastAim，且重验所有 dash 时序。]),
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
