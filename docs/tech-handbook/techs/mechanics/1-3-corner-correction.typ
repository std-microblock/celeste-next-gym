#import "../../template.typ": tech, evidence

#tech(
  id: "1.3",
  title-zh: "墙角修正",
  title-en: "Corner Correction",
  status: "unimplemented",
  description-zh: [水平冲刺撞到墙角，或在天花板边缘 4 像素范围内接触时，游戏会尝试把玩家沿墙角推出，避免被边缘直接截停。],
  description-en: [Near a solid corner, horizontal dashes and ceiling contacts can be shifted by up to four pixels so the player clears the edge instead of stopping.],
  source-evidence: none,
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
