#import "../../template.typ": tech, evidence

#tech(
  id: "3.7.11",
  title-zh: "消失方块 Cornerboost",
  title-en: "Disappearing Block Cornerboost",
  status: "implemented",
  description-zh: [玩家水平撞上门、卡带方块等 Solid 后会保存墙速；若实体在 0.06 秒窗口内解除 Collidable，下一次 Player.Update 会发现前方已空并返还 retained speed。卡带 manager 在碰撞后的实体阶段改 Activated，CassetteBlock 随即清除 Collidable，因此下一次 Player.Update 返还速度。],
  description-en: [A horizontal Solid collision stores wall speed. If a door or cassette block becomes non-collidable inside the 0.06-second window, the next Player.Update finds the path clear and restores the retained speed. The cassette manager changes Activated in the post-collision entity phase and CassetteBlock then clears Collidable, so the following Player.Update refunds the speed.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Source/Entities/CassetteBlock.cs],
    symbol: [Player.OnCollideH; Player.Update; CassetteBlock.Update],
    snippet: raw(block: true, lang: "cs", "wallSpeedRetained = Speed.X;\nwallSpeedRetentionTimer = WallSpeedRetentionTime;\n...\nif (!CollideCheck<Solid>(Position + Vector2.UnitX * Math.Sign(wallSpeedRetained)))\n    Speed.X = wallSpeedRetained;\n...\nif (Activated != Collidable) Collidable = Activated;"),
    note: [OnCollideH 在把 Speed.X 清零前保存速度；每次 Player.Update 都先探测 retained 方向的一像素。CassetteBlock 在碰撞后的实体阶段随 beat 切换 Collidable，因此下一玩家帧可以命中返还分支。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [update_wall_speed_retention; advance_cassette_manager; advance_cassette_blocks], note: [Player 先于房间实体更新；同一 post-player entity phase 先推进 cassette manager，再由 block 读新的 Activated 并 park 为非碰撞。下一玩家帧的 retained-speed 探测因此看见前方为空。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [disappearing_cassette_cornerboost_restores_retained_speed_after_entity_phase; disappearing_cassette_cornerboost_fixture_times_hit_clear_and_refund], note: [回归锁定 fresh manager 的 tempo=3：state 28 撞墙、速度归零且 index-1 在同一 post-player phase 失活/不可碰撞；state 29 在仍未到期的 0.06 秒窗口恢复为 `(126,496)/(90,0)`。]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/dashless-3.7.11-disappearing-block-cornerboost.ts; scripts/e2e-real/scenarios/cassette-spinner-parts.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs; .tmp/e2e-runs/2026-07-28T18-33-46.082Z-112388-7e2cda87-7151-4704-aa7d-fd48bba2faf9/manifest.json], symbol: [dashless-3.7.11-disappearing-block-cornerboost; TECH_DASHLESS_3_7_11_DISAPPEARING_BLOCK_CORNERBOOST; SnapshotCapture.Capture], note: [2026-07-28 隔离真实 Everest run 完成物理 `vendor/celeste-game` 校验、isolated save/tmp、动态端口 49765/49766、nonce 和本次 spawned Celeste PID 112956 的精确握手及受控清理。61 帧 position、speed、state、facing、dashes、stamina、grounded、ducking、death 均逐帧匹配，position/speed 最大误差均为 0。真实轨迹 state 28 在 `(124,496)/(0,0)` 碰墙且 index-1 已非碰撞，state 29 恢复为 `(126,496)/(90,0)`。]),
  candidate-e2e: none,
)
