#import "../../template.typ": tech, evidence

#tech(
  id: "1.5",
  title-zh: "冲刺攻击窗口",
  title-en: "Dash Attack",
  status: "unimplemented",
  description-zh: [冲刺结束后仍保留约 6 帧 Dash Attack，可触发墙反、冲刺开关等交互；跳跃、攀跳、墙跳或抓取会提前取消它。],
  description-en: [A roughly six-frame dash-attack window survives after dash movement ends, enabling dash interactions until a jump, wall action, or grab cancels it.],
  source-evidence: none,
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
