#import "../../template.typ": tech, evidence

#tech(
  id: "1.3",
  title-zh: "墙角修正",
  title-en: "Corner Correction",
  status: "implemented",
  description-zh: [水平冲刺撞到墙角，或在天花板边缘 4 像素范围内接触时，游戏会尝试把玩家沿墙角推出，避免被边缘直接截停。],
  description-en: [Near a solid corner, horizontal dashes and ceiling contacts can be shifted by up to four pixels so the player clears the edge instead of stopping.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.OnCollideH; Player.OnCollideV],
    snippet: raw(block: true, lang: "cs", "for (int i = 1; i <= DashCornerCorrection; i++) {\n    for (int j = 1; j >= -1; j -= 2)\n        if (!CollideCheck<Solid>(Position + new Vector2(Math.Sign(Speed.X), i * j))) {\n            MoveVExact(i * j);\n            MoveHExact(Math.Sign(Speed.X));\n            return;\n        }\n}\n...\nfor (int i = 1; i <= UpwardCornerCorrection; i++)\n    if (!CollideCheck<Solid>(Position + new Vector2(i, -1))) {\n        Position += new Vector2(i, -1);\n        return;\n    }"),
    note: [水平 Dash/RedDash 撞墙时按上、下顺序尝试 1–4 像素绕角；向上碰顶则依据水平速度符号尝试左右 1–4 像素。两个分支都在停止速度前执行，因此成功修正会保留原移动速度。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [move_axis_amount]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [horizontal_dash_corner_correction_moves_over_a_two_pixel_ledge_overlap; upward_corner_correction_moves_around_a_one_pixel_ceiling_overlap]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [mechanics-corner-correction-horizontal; mechanics-corner-correction-up], note: [水平场景第 5 帧位于 396/82，第 6 帧绕过两像素墙角到 397/80 并保留 240 水平速度；上顶场景第 1 帧从 477/275 修正到 476/274。两场景九类字段逐帧一致，max position error 0，max speed error 0。]),
  candidate-e2e: none,
)
