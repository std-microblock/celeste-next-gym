#import "../../template.typ": tech, evidence

#tech(
  id: "2.8.2.1",
  title-zh: "贴地 Ultra 取消",
  title-en: "Grounded Ultra Cancel",
  status: "unimplemented",
  description-zh: [用抓取投掷物、跳过过场或弹跳等方式提前打断贴地 Ultra，可以绕过冲刺结束时的速度重置。],
  description-en: [Interrupting a grounded ultra before dash end, for example with a grab or bounce, preserves speed that the normal dash exit would remove.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.DashUpdate; Player.OnCollideV],
    snippet: raw(block: true, lang: "cs", "if (Holding == null && DashDir != Vector2.Zero && Input.Grab.Check && !IsTired && CanUnDuck) {\n    if (hold.Check(this) && Pickup(hold)) return StPickup;\n}\n// Grounded ultra speed is otherwise normalized when Dash ends."),
    note: [源码确认 Dash 可被拾取等状态切换提前打断；但当前模拟器尚无能在贴地 Ultra 精确窗口触发的真实取消实体/机制，因此没有 Rust 回归或有效 E2E，继续明确保持未实现。],
  ),
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
