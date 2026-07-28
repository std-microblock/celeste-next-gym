#import "../../template.typ": tech, evidence

#tech(
  id: "3.7.11",
  title-zh: "消失方块 Cornerboost",
  title-en: "Disappearing Block Cornerboost",
  status: "unimplemented",
  description-zh: [玩家水平撞上门、卡带方块等 Solid 后会保存墙速；若实体在 0.06 秒窗口内解除 Collidable，下一次 Player.Update 会发现前方已空并返还 retained speed。卡带 manager 在 Player 后才改 Activated，CassetteBlock 又在下一实体帧才真正清除 Collidable，因此返还发生在碰墙后的第三次 Player.Update。],
  description-en: [A horizontal Solid collision stores wall speed. If a door or cassette block becomes non-collidable inside the 0.06-second window, the next Player.Update finds the path clear and restores the retained speed. The manager changes Activated after Player, while CassetteBlock clears Collidable on the next entity phase, so the refund occurs on the third Player.Update after impact.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Source/Entities/CassetteBlock.cs],
    symbol: [Player.OnCollideH; Player.Update; CassetteBlock.Update],
    snippet: raw(block: true, lang: "cs", "wallSpeedRetained = Speed.X;\nwallSpeedRetentionTimer = WallSpeedRetentionTime;\n...\nif (!CollideCheck<Solid>(Position + Vector2.UnitX * Math.Sign(wallSpeedRetained)))\n    Speed.X = wallSpeedRetained;\n...\nif (Activated != Collidable) Collidable = Activated;"),
    note: [OnCollideH 在把 Speed.X 清零前保存速度；每次 Player.Update 都先探测 retained 方向的一像素。CassetteBlock 随 beat 切换 Collidable，因此实体更新后的下一玩家帧可以命中返还分支。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [update_wall_speed_retention; advance_cassette_blocks; advance_cassette_manager], note: [Player 在房间实体前更新；卡带 block 更新再到 manager beat 更新的顺序保持 WillToggle 与 activation 分帧，非碰撞 block 会被 park，下一帧 retained-speed 探测可见前方为空。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [disappearing_cassette_cornerboost_restores_retained_speed_after_entity_phase], note: [回归让 index-0 cassette 在第一帧挡住 120 速度并记录 retained speed；同帧 beat 8 只写 `Activated=false`，第二帧实体 phase 才清除碰撞，第三个 Player.Update 在仍未到期的 0.06 秒窗口返还速度，常规空中减速后仍大于 90。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/dashless-3.7.11-disappearing-block-cornerboost.ts; scripts/e2e-real/scenarios/cassette-spinner-parts.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs], symbol: [dashless-3.7.11-disappearing-block-cornerboost; TECH_DASHLESS_3_7_11_DISAPPEARING_BLOCK_CORNERBOOST; SnapshotCapture.Capture], note: [候选 MapPart 已把靠近玩家的竖墙设为初始 active 的 index 1，输入延后至首个 8-beat 切色前；collector 逐帧采集 cassette index、位置、Activated 和 Collidable，场景按“撞墙→index 1 消失→速度返还”验证。尚未运行新的真实 Everest trace，故保持 candidate。]),
)
