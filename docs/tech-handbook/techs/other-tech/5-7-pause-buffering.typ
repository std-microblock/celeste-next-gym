#import "../../template.typ": tech, evidence

#tech(
  id: "5.7",
  title-zh: "暂停缓冲",
  title-en: "Pause Buffering",
  status: "unimplemented",
  description-zh: [解除暂停前约 6 帧输入动作，可让它在游戏恢复的第一帧执行；重复暂停还能近似逐帧控制。],
  description-en: [Inputs made up to about six frames before unpausing execute on the first active frame and can be paired with repeated pauses for frame stepping.],
  source-evidence: none,
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
