#import "../../template.typ": tech, evidence

#tech(
  id: "4.18.2.1",
  title-zh: "Cassoosted Fuper",
  title-en: "Cassoosted Fuper",
  status: "unimplemented",
  description-zh: [在执行 Feather Super 的同一时机获得 Cassette Reform Boost，把两种技巧的长跳和纵向加速组合起来。],
  description-en: [This composition requires both the CassetteBlock reform wiggle/ShiftSize lifecycle and a feather super on the same frame. Feather Super is implemented, but the global cassette runtime is absent, so Cassoosted Fuper remains unimplemented.],
  source-evidence: evidence(
    path: [Celeste/CassetteBlock.cs; Source/Player/Player.cs],
    symbol: [CassetteBlock.Update / Player.StarFlyEnd / Player.SuperJump],
    snippet: raw(block: true, lang: "cs", "Collidable = true;\nEnableStaticMovers();\nShiftSize(-1);\n...\nSpeed.X = 260f * (int)Facing;\nSpeed.Y = -105f;\nSpeed += LiftBoost;"),
    note: [组合要求 Cassette 同帧重组位移先产生 lift，再让 StarFly 退出后的 SuperJump 叠加该 LiftBoost；缺少 cassette beat/group runtime 时不能只用已实现的 Feather Super 宣称闭环。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [feather_super_jumps_from_grounded_horizontal_starfly_speed], note: [Feather Super 组成部分已有回归，但 Cassette 组成部分缺失。]),
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
