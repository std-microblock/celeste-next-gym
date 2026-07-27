#import "../../template.typ": tech, evidence

#tech(
  id: "5.11",
  title-zh: "Spinner 长时冻结",
  title-en: "Spinner Freeze",
  status: "unimplemented",
  description-zh: [关卡运行约 118 小时后，浮点精度使 TimeActive 停止递增，导致三组 Spinner 中有两组永远不再执行碰撞启用检查。],
  description-en: [After roughly 118 hours, floating-point precision can freeze TimeActive so two of three spinner groups stop receiving activation checks.],
  source-evidence: none,
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
