#import "../../template.typ": tech, evidence

#tech(
  id: "4.22",
  title-zh: "中性放下",
  title-en: "Neutral Drop",
  status: "unimplemented",
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
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.22-neutral-drop.ts], symbol: [entity-4.22-neutral-drop], note: [独立 MapPart 验证松抓帧玩家水平速度为零、CannotHold 阻止立即重抓且 Theo 留在原位；真实 Everest 尚待 FIFO 锁内采集。]),
)
