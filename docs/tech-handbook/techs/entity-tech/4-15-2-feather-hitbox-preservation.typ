#import "../../template.typ": tech, evidence

#tech(
  id: "4.15.2",
  title-zh: "羽毛碰撞箱保留",
  title-en: "Feather Hitbox Preservation",
  status: "unimplemented",
  description-zh: [用冰球、无敌反弹等特殊方式取消羽毛，可留下异常的碰撞箱与受伤箱组合，用于穿越原本不可能的空间。],
  description-en: [Special feather cancellations can preserve unusual collider and hurtbox combinations that fit or survive in otherwise impossible spaces.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Source/FireBall.cs],
    symbol: [Player.Bounce / Player.StarFlyEnd / FireBall.OnBounce],
    note: [Player.Update 在 PlayerCollider 阶段先把 Collider 临时指向当前 hurtbox；Ice Ball 的 OnBounce 就在该循环中调用 Bounce。Bounce 因而缓存 6×6 StarFly hurtbox，再切回 Normal 触发 StarFlyEnd 恢复普通 hurtbox，最后把缓存对象写回 Collider，形成 6×6 collider + 8×9 normal hurtbox。],
    snippet: raw(block: true, lang: "cs", "Collider collider = Collider;\nCollider = hurtbox;\nforeach (PlayerCollider pc in Scene.Tracker.GetComponents<PlayerCollider>())\n    pc.Check(this);\n...\nvar was = Collider;\nCollider = normalHitbox;\nStateMachine.State = StNormal; // runs StarFlyEnd\nCollider = was;"),
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [interact / bounce / star_fly_hurt_rect / current_player_rect / star_fly_hitbox_preserved], note: [移动后同帧执行 FireBall PlayerCollider bounce；Normal 状态下 preservation 标记选择 6×6 rect，上升结束转为下落时按 Player.Update 的 falling-unduck 顺序恢复普通 collider。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [ice_ball_same_frame_callback_keeps_split_simulation_composable / ice_ball_feather_cancel_restores_star_fly_collider_after_normal_hurtbox / preserved_star_fly_hurtbox_returns_to_normal_when_falling / playground_feather_cancel_scenario_preserves_the_star_fly_collider]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.15.2-feather-hitbox-preservation.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs], symbol: [entity-4.15.2-feather-hitbox-preservation / ColliderGeometry], note: [独立 Feather + Ice Ball MapPart 已断言 Bounce 后真实几何为更新期缓存的 StarFly hurtbox collider (-3,-9,6,6) 与 normal hurtbox (-4,-11,8,9)；等待真实 Everest 九字段差分。]),
)
