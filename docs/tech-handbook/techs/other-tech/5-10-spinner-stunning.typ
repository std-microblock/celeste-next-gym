#import "../../template.typ": tech, evidence

#tech(
  id: "5.10",
  title-zh: "Spinner 暂停眩晕",
  title-en: "Spinner Stunning",
  status: "unimplemented",
  description-zh: [Spinner 每 3 帧检查一次是否应启用碰撞；在对应检查帧暂停可跳过检查，并可反复延长无碰撞状态。],
  description-en: [Spinners update proximity collision every third frame, and pausing on their check frame can skip activation and keep them harmless longer.],
  source-evidence: none,
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
