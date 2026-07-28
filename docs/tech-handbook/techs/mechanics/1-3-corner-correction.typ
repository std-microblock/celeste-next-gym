#import "../../template.typ": tech, evidence

#tech(
  id: "1.3",
  title-zh: "墙角修正",
  title-en: "Corner Correction",
  status: "implemented",
  description-zh: [水平冲刺撞到墙角、向下冲刺擦到地板边缘，或在天花板边缘 4 像素范围内接触时，游戏会尝试把玩家沿墙角推出，避免被边缘直接截停。],
  description-en: [Near a solid corner, horizontal dashes, airborne downward dashes grazing a floor edge, and ceiling contacts can be shifted by up to four pixels so the player clears the edge instead of stopping.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.OnCollideH; Player.OnCollideV],
    snippet: raw(block: true, lang: "cs", "// OnCollideH\nfor (int i = 1; i <= DashCornerCorrection; i++)\n    for (int j = 1; j >= -1; j -= 2)\n        if (!CollideCheck<Solid>(Position + new Vector2(Math.Sign(Speed.X), i * j))) {\n            MoveVExact(i * j); MoveHExact(Math.Sign(Speed.X)); return;\n        }\n\n// OnCollideV, Speed.Y > 0\nif ((StateMachine.State == StDash || StateMachine.State == StRedDash) && !dashStartedOnGround) {\n    if (Speed.X <= 0)\n        for (int i = -1; i >= -DashCornerCorrection; i--)\n            if (!OnGround(Position + new Vector2(i, 0))) {\n                MoveHExact(i); MoveVExact(1); return;\n            }\n    if (Speed.X >= 0) { /* mirror the search to the right */ }\n}"),
    note: [水平 Dash/RedDash 撞墙时按上、下顺序尝试 1–4 像素绕角。空中开始的向下 Dash/RedDash 碰地时，先按 `Speed.X` 允许的方向搜索 1–4 像素；纯竖直速度先搜左再搜右，成功后水平移动并继续下移 1 像素。修正发生在落地清速之前，因此保留冲刺速度；`dashStartedOnGround` 会阻止地面起冲使用该分支。向上碰顶另按水平速度符号尝试左右 1–4 像素。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [move_axis_amount; grounded_at_position]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [horizontal_dash_corner_correction_moves_over_a_two_pixel_ledge_overlap; upward_corner_correction_moves_around_a_one_pixel_ceiling_overlap; downward_dash_corner_correction_moves_left_around_a_one_pixel_floor_overlap; downward_dash_corner_correction_follows_horizontal_speed_direction; downward_dash_started_on_ground_does_not_corner_correct]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/mechanics-corner-correction-horizontal.ts; scripts/e2e-real/scenarios/playground/mechanics-corner-correction-up.ts; scripts/e2e-real/scenarios/playground/mechanics-corner-correction-down.ts], symbol: [mechanics-corner-correction-horizontal; mechanics-corner-correction-up; mechanics-corner-correction-down], note: [水平场景第 5 帧位于 396/82，第 6 帧绕过两像素墙角到 397/80 并保留 240 水平速度；上顶场景第 1 帧从 477/275 修正到 476/274。2026-07-28 隔离真实 Everest run `2026-07-28T14-46-40.716Z-107052-b866a502-32d8-477f-84e2-5ba5738abaa2` 中，向下场景第 8 帧位于 251/112，第 9 帧向右并向下各修正 1 像素到 252/113，保持速度 0/240。13 个状态的九类核心字段逐帧一致，max position error 0，max speed error 0。]),
  candidate-e2e: none,
)
