#import "../../template.typ": tech, evidence

#tech(
  id: "3.12.2",
  title-zh: "Wallboost Neutral",
  title-en: "Wallboost Neutral",
  status: "implemented",
  description-zh: [Wallboost 后立刻转回墙面并重复，可像 neutral 墙跳一样无限攀升而不消耗体力。],
  description-en: [Turning back toward the wall after each wallboost allows repeated stamina-free climbing similar to neutral jumps.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.NormalUpdate; Player.ClimbBegin; Player.ClimbJump],
    snippet: raw(block: true, lang: "cs", "if (Speed.Y >= 0 && Math.Sign(Speed.X) != -(int) Facing) {\n    if (ClimbCheck((int) Facing))\n        return StClimb;\n}\n...\nfor (int i = 0; i < ClimbCheckDist; i++)\n    if (!CollideCheck<Solid>(Position + Vector2.UnitX * (int) Facing))\n        Position += Vector2.UnitX * (int) Facing;\n...\nif (moveX == 0) {\n    wallBoostDir = -(int) Facing;\n    wallBoostTimer = ClimbJumpBoostTime;\n}"),
    note: [Wallboost 后转回墙面，NormalUpdate 只在下落、未正离墙且 Facing 方向 2px 内有墙时重新抓墙。ClimbBegin 会贴近一像素缝隙；再次 neutral climb jump 会打开新的 wallboost 窗口，形成不消耗体力的重复循环。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [normal_update; climb_bounds_check; climb_check; slip_check; climb_update; update_wall_boost]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [wallboost_neutral_returns_to_the_wall_for_a_second_stamina_free_cycle; climb_begin_at_a_ledge_uses_slip_speed_during_the_no_move_window]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [wallboost-neutral], note: [标准墙面上完成两次 neutral climb jump、wallboost 与回墙循环；61 个状态帧九字段逐帧一致，max position error 0，max speed error 0.000069。]),
  candidate-e2e: none,
)
