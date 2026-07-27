#import "../../template.typ": tech, evidence

#tech(
  id: "3.9",
  title-zh: "尖刺穿越",
  title-en: "Spike Clip",
  status: "unimplemented",
  description-zh: [尖刺只检测受伤箱底部像素；速度足够高时可让该像素在相邻帧从尖刺上方直接越到下方而不触发死亡。],
  description-en: [At sufficient speed, the bottom hurtbox pixel can move from above unsupported spikes to below them between frames without intersecting their lethal region.],
  source-evidence: none,
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
