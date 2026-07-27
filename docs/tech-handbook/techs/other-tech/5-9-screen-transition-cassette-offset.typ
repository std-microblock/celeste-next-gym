#import "../../template.typ": tech, evidence

#tech(
  id: "5.9",
  title-zh: "房间切换卡带偏移",
  title-en: "Screen Transition Cassette Offset",
  status: "unimplemented",
  description-zh: [卡带 beat 切色与跨房实体加载同帧发生时，上一颜色和当前颜色 CassetteBlock 的启停位移会落在不同更新阶段，造成一组下移 1 像素、另一组上移 1 像素。当前 Rust 地图与实体模型没有卡带全局时钟、颜色组或 CassetteBlock，故只保留机制审计。],
  description-en: [When a cassette beat changes color on the same frame as a room transition, previous- and current-color CassetteBlocks apply activation displacement in different update phases, producing opposite one-pixel offsets. The Rust map/entity model has no cassette clock, color groups, or CassetteBlock, so this remains a mechanism audit.],
  source-evidence: evidence(path: [Celeste/Level.cs; Celeste/CassetteBlockManager.cs; Celeste/CassetteBlock.cs], symbol: [Level.TransitionRoutine; CassetteBlockManager.AdvanceMusic; CassetteBlock.SetActivated], note: [TransitionRoutine 在相机切换与新房实体装载之间推进关卡；CassetteBlockManager 按 beat 切换当前 index，CassetteBlock.SetActivated 通过一像素纵向位移改变碰撞。技巧依赖新旧房实体加入 Scene 的精确帧与颜色组激活顺序。]),
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
