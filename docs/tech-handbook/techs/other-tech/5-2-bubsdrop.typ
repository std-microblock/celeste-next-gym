#import "../../template.typ": tech, evidence

#tech(
  id: "5.2",
  title-zh: "Bubsdrop 回房下落",
  title-en: "Bubsdrop",
  status: "unimplemented",
  description-zh: [在向上房间切换时用墙跳或攀跳取消上升动量，避免落上单向平台并掉回旧房间，从而触发新的出生点选择。],
  description-en: [A wallkick or climb jump cancels upward transition momentum so the player misses a jumpthrough, returns to the prior room, and receives a different spawn.],
  source-evidence: none,
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
