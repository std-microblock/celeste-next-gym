#import "../../template.typ": tech, evidence

#tech(
  id: "2.9",
  title-zh: "Demo 蹲冲",
  title-en: "Demodash (Demo)",
  status: "implemented",
  description-zh: [以蹲伏碰撞箱进行冲刺，角色高度从普通的 9 像素降为 4 像素，可穿过更窄的危险间隙。],
  description-en: [A demodash uses the four-pixel crouched hitbox during a dash, allowing passage through gaps that reject the normal nine-pixel hitbox.],
  source-evidence: evidence(
    path: [vendor/celeste-game/Celeste.dll（Everest patched Player）],
    symbol: [Player.StartDash; Player.DashBegin; Player.Ducking],
    snippet: raw(block: true, lang: "cs", "demoDashed = Input.CrouchDashPressed;\n...\nelse if (!Ducking && (demoDashed || Input.MoveY.Value == 1)) {\n    Ducking = true;\n}\n// duck hurtbox: 8x4 at (-4,-6)"),
    note: [Everest 的 CrouchDash 输入在 StartDash 记录 demoDashed，DashBegin 在冲刺位移前切换到蹲伏碰撞箱；伤害箱从普通 8×9 降为 8×4。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [InputState.crouch_dash_pressed; PlayerSnapshot.demo_dashed; begin_dash; current_player_hurt_rect]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [demodash_passes_a_six_pixel_gap_that_blocks_a_normal_dash], note: [同一条 8 像素瓦片隧道中，蹲冲的 8×6 碰撞箱通过，而普通冲刺被低顶阻挡。]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [dash-demodash-gap; verifyDemodashGap], note: [真实 31 帧蹲冲场景以 ducking 状态进入六像素隧道并存活穿过至 X=778；九类核心字段最大位置与速度误差均为 0。]),
  candidate-e2e: none,
)
