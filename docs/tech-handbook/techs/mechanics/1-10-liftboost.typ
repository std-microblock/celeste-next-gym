#import "../../template.typ": tech, evidence

#tech(
  id: "1.10",
  title-zh: "平台动量加成",
  title-en: "Liftboost",
  status: "unimplemented",
  description-zh: [移动实体赋予的位移速度会保存为 liftboost；站立、抓墙、跳跃或冲刺时按各自动作规则叠加，并在离开实体后短暂保留。],
  description-en: [Movement inherited from a moving entity is stored as liftboost and applied by standing, grabbing, jumping, and dashing rules for a short grace period.],
  source-evidence: none,
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
