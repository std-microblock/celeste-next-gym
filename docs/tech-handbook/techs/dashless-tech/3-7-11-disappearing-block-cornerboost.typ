#import "../../template.typ": tech, evidence

#tech(
  id: "3.7.11",
  title-zh: "消失方块 Cornerboost",
  title-en: "Disappearing Block Cornerboost",
  status: "unimplemented",
  description-zh: [玩家水平撞上门、卡带方块等 Solid 后会保存墙速；若实体在 0.06 秒窗口内解除 Collidable，下一次 Player.Update 会发现前方已空并返还 retained speed。Rust 已具备卡带消失生命周期和精确回归，但完整真实候选尚待 Everest 验证。],
  description-en: [A horizontal Solid collision stores wall speed. If a door or cassette block becomes non-collidable inside the 0.06-second window, the next Player.Update finds the path clear and restores the retained speed. Rust has the cassette disappearance lifecycle and a regression; the complete candidate still awaits real Everest validation.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Source/Entities/CassetteBlock.cs],
    symbol: [Player.OnCollideH; Player.Update; CassetteBlock.Update],
    snippet: raw(block: true, lang: "cs", "wallSpeedRetained = Speed.X;\nwallSpeedRetentionTimer = WallSpeedRetentionTime;\n...\nif (!CollideCheck<Solid>(Position + Vector2.UnitX * Math.Sign(wallSpeedRetained)))\n    Speed.X = wallSpeedRetained;\n...\nif (Activated != Collidable) Collidable = Activated;"),
    note: [OnCollideH 在把 Speed.X 清零前保存速度；每次 Player.Update 都先探测 retained 方向的一像素。CassetteBlock 随 beat 切换 Collidable，因此实体更新后的下一玩家帧可以命中返还分支。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [update_wall_speed_retention; advance_cassette_blocks; advance_cassette_manager], note: [Player 在房间实体前更新；卡带 block 更新再到 manager beat 更新的顺序保持 WillToggle 与 activation 分帧，非碰撞 block 会被 park，下一帧 retained-speed 探测可见前方为空。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [disappearing_cassette_clears_collision_before_reactivation], note: [精确回归从 beat 6 推进到 7/8，断言目标 block 在警告 beat 保持非碰撞，再在下一色 activation 时恢复。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/dashless-3.7.11-disappearing-block-cornerboost.ts; scripts/e2e-real/scenarios/cassette-spinner-parts.ts], symbol: [dashless-3.7.11-disappearing-block-cornerboost; TECH_DASHLESS_3_7_11_DISAPPEARING_BLOCK_CORNERBOOST], note: [独立双颜色 CassetteBlock MapPart 让玩家在 beat 窗口撞上竖直 block，并要求消失后重新获得向右速度；真实帧序未通过前保持 candidate。]),
)
