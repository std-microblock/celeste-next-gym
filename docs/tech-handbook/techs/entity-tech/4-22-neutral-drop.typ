#import "../../template.typ": tech, evidence

#tech(
  id: "4.22",
  title-zh: "中性放下",
  title-en: "Neutral Drop",
  status: "implemented",
  description-zh: [按住下方向并松开抓取，会让投掷物原地落下而不获得水平投掷速度。],
  description-en: [Holding down while releasing grab drops a holdable with no horizontal throw speed.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Source/Holdable.cs / Source/TheoCrystal.cs / Source/Glider.cs],
    symbol: [Player.Throw / Player.Drop / Holdable.Release / TheoCrystal.OnRelease / Glider.OnRelease],
    snippet: raw(block: true, lang: "cs", "if (Input.MoveY.Value == 1) Drop();\n...\nHolding.Release(Vector2.Zero);\n...\nSpeed = force * 200f;"),
    note: [下方向让 Throw 分派到 Drop，并以 Vector2.Zero 调用通用 Holdable.Release；Theo 的 200 倍与 Glider 的 100 倍都仍为零，因此没有水平投掷速度或玩家 ThrowRecoil，仅保留 CannotHold／gravity 窗口。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [release_theo / normal_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [neutral_drop_releases_theo_without_throw_speed_or_player_recoil]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.22-neutral-drop.ts], symbol: [entity-4.22-neutral-drop], note: [独立 Theo MapPart 的真实 Everest 轨迹依次观测 pickup、零水平 recoil 的 neutral drop、六帧不可立即重抓与原位 regrab。49 个状态的 position、speed、state、facing、dashes、stamina、grounded、ducking、death 全部逐帧一致，最大 position／speed 误差均为 0。]),
  candidate-e2e: none,
)
