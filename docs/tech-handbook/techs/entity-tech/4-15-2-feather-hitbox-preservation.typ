#import "../../template.typ": tech, evidence

#tech(
  id: "4.15.2",
  title-zh: "羽毛碰撞箱保留",
  title-en: "Feather Hitbox Preservation",
  status: "unimplemented",
  description-zh: [用冰球、无敌反弹等特殊方式取消羽毛，可留下异常的碰撞箱与受伤箱组合，用于穿越原本不可能的空间。],
  description-en: [Special feather cancellations can preserve unusual collider and hurtbox combinations that fit or survive in otherwise impossible spaces.],
  source-evidence: evidence(path: [Source/Player/Player.cs; Source/FireBall.cs], symbol: [Player.Bounce / Player.StarFlyEnd / FireBall.OnBounce], note: [Bounce 先缓存当前 Collider，再把状态切回 Normal；StarFlyEnd 在状态回调中恢复普通 collider/hurtbox，随后 Bounce 又写回缓存的 starFlyHitbox，因此形成 StarFly collider + normal hurtbox。]),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [bounce / end_star_fly / current_player_rect / star_fly_hitbox_preserved]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [ice_ball_feather_cancel_restores_star_fly_collider_after_normal_hurtbox / playground_feather_cancel_scenario_preserves_the_star_fly_collider]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [entity-4.15.2-feather-hitbox-preservation], note: [Feather 向下飞行顶踩静止 Ice Ball，待隔离 Celeste 实测九字段。]),
)
