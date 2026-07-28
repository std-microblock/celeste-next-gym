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
    note: [NormalUpdate 先处理松抓／Drop，再允许同帧 StartDash；后续 DashUpdate 抓到物品会以 PickupCoroutine 保存并恢复抓取瞬间的直线 Dash 速度。完整 Slash 还要求带纵速的中性放下与实际穿越重抓轨迹，当前候选尚未稳定形成。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [release_theo / normal_update / dash_update / pickup_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [neutral_drop_can_start_a_dash_on_the_same_normal_update]),
  e2e-evidence: none,
  candidate-e2e: none,
)
