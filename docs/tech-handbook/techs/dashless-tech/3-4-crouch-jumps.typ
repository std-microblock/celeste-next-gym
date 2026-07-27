#import "../../template.typ": tech, evidence

#tech(
  id: "3.4",
  title-zh: "蹲伏跳",
  title-en: "Crouch Jumps",
  status: "unimplemented",
  description-zh: [蹲伏时起跳会在上升阶段继续使用矮碰撞箱，直到角色下落或上方空间允许站起。],
  description-en: [Jumping while crouched preserves the short hitbox while rising or while there is not enough room to stand.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.NormalUpdate; Player.Update; Player.CanUnDuck; Player.Jump],
    snippet: raw(block: true, lang: "cs", "else if (onGround && Input.MoveY == 1 && Speed.Y >= 0)\n    Ducking = true;\n...\nif (jumpGraceTimer > 0)\n    Jump();\n...\nif (Speed.Y > 0 && CanUnDuck && Collider != starFlyHitbox && !onGround)\n    Ducking = false;"),
    note: [普通 Jump 不会清除 Ducking，因此从蹲伏起跳后整个上升阶段继续使用 8×6 矮碰撞箱。只有进入下落、离地且普通 8×11 碰撞箱无阻挡时，Player.Update 才恢复站立。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [step; normal_update; current_player_rect; can_unduck],
    note: [模拟器保留蹲伏跳的短碰撞箱，并在移动前仅于下落、离地且 can_unduck 成立时恢复普通碰撞箱。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [crouch_jump_keeps_the_short_hitbox_until_falling_in_open_air],
    note: [测试覆盖接地蹲伏、蹲伏起跳、全上升阶段保持矮碰撞箱、开放空间下落恢复站立，以及低天花板下落时因 CanUnDuck 失败继续蹲伏。],
  ),
  e2e-evidence: none,
  candidate-e2e: "crouch-jump",
)
