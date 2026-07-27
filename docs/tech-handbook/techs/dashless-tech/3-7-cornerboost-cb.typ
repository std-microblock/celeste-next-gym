#import "../../template.typ": tech, evidence

#tech(
  id: "3.7",
  title-zh: "Cornerboost 墙角加速",
  title-en: "Cornerboost (cb)",
  status: "unimplemented",
  description-zh: [撞墙时保存的 retained speed 会在 5 帧内脱离阻挡后返还；在墙角攀跳可同时获得跳跃加速并取回原速度。],
  description-en: [Wall collision stores retained speed for five frames; climb-jumping past the corner refunds it while adding jump acceleration.],
  source-evidence: none,
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
