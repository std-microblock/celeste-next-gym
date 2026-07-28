#import "../../template.typ": tech, evidence

#tech(
  id: "4.21",
  title-zh: "携物 Slash",
  title-en: "Holdable Slash",
  status: "unimplemented",
  description-zh: [中性放下投掷物后，以足够纵向速度水平冲刺并重新抓取，可取消冲刺并保留直线动量。],
  description-en: [After a neutral drop at suitable vertical speed, dash horizontally into the holdable and regrab it to preserve the straight dash momentum.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Source/Holdable.cs],
    symbol: [Player.NormalUpdate / Player.Drop / Player.DashUpdate / Player.PickupCoroutine],
    snippet: raw(block: true, lang: "cs", "if (Input.MoveY.Value == 1) Drop();\n...\nif (CanDash) return StartDash();\n...\nSpeed = oldSpeed;\nSpeed.Y = Math.Min(Speed.Y, 0);"),
    note: [NormalUpdate 先处理松抓／Drop，再允许同帧 StartDash；后续 DashUpdate 抓到物品会以 PickupCoroutine 保存并恢复抓取瞬间的直线 Dash 速度。候选场景把这些顺序放在独立 Theo MapPart 中，并要求放下时仍在空中且带非零纵速。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [release_theo / normal_update / dash_update / pickup_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [holdable_slash_regrabs_theo_in_horizontal_dash_with_airborne_vertical_speed]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.21-holdable-slash.ts], symbol: [entity-4.21-holdable-slash], note: [独立候选要求空中中性放下、水平 Dash 穿回 Theo、Pickup 重抓与直线 240 速度；真实 Everest 尚待九字段对照，因此保持未实现。]),
)
