#import "../../template.typ": tech, evidence

#tech(
  id: "2.8.2",
  title-zh: "贴地 Ultra",
  title-en: "Grounded Ultras",
  status: "unimplemented",
  description-zh: [在地面或极近地面处向下斜冲刺会立即获得 Ultra 倍率，但通常会在冲刺自然结束时失去多余速度。],
  description-en: [A down-diagonal dash on or very near the floor gains the ultra multiplier immediately, but excess speed is normally removed when the dash ends.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.DashCoroutine; Player.OnCollideV; Player.DashUpdate],
    snippet: raw(block: true, lang: "cs", "Speed = DashDir * DashSpeed;\n...\nif (DashDir.X != 0 && DashDir.Y > 0 && Speed.Y > 0) {\n    Speed.Y = 0;\n    Speed.X *= 1.2f;\n    Ducking = true;\n}"),
    note: [贴地向下斜冲在冲刺速度刚赋值后立即进入垂直碰撞分支，因此同帧取得 1.2 倍；若不提前打断，DashUpdate 的正常结束仍会把速度收束到冲刺结束值。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [dash_update; move_axis_amount]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [grounded_ultra_preserves_faster_entry_speed_before_multiplier], note: [回归以 300 水平入速开始贴地向下斜冲，证明先保留更快同向入速，再在触地分支乘 1.2。]),
  e2e-evidence: none,
  candidate-e2e: "dash-grounded-ultra",
)
