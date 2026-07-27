#import "../../template.typ": tech, evidence

#tech(
  id: "4.9",
  title-zh: "Dream Grab 梦块抓墙",
  title-en: "Dream Grab",
  status: "implemented",
  description-zh: [离开梦块时按住抓取并输入反方向，可以抓住刚离开的梦块侧面。],
  description-en: [Holding grab and the opposite direction on dream-block exit lets the player catch the block's outer wall.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.DreamDashUpdate / Player.DreamDashEnd],
    note: [横向离开 DreamBlock 时，原版先按冲刺方向向块内回拉最多 5 像素，再在出口帧执行两侧 `ClimbCheck`；抓取键与朝墙输入匹配时立即进入 Climb。DreamDashEnd 随后恢复横向出口的 0.1 秒 jump grace、dash 和 stamina。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [interact],
    note: [DreamDash 出口按原版顺序执行 5 像素静态 Solid 修正、wall_dir、grab/input 判定，并在同帧切换 Climb 与恢复出口资源。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [dream_grab_catches_the_block_wall_on_the_exit_frame / dream_grab_uses_v14_five_pixel_static_solid_correction],
  ),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [entity-4.9-dream-grab],
    note: [原版第二章 `lvl_1` 场景共 29 个状态；position/speed 最大误差均为 0，state、facing、dashes、stamina、grounded、ducking、death 全部一致。终态为 Climb、面向左，确认出口抓墙语义实际命中。],
  ),
  candidate-e2e: none,
)
