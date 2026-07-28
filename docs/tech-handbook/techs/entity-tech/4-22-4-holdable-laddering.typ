#import "../../template.typ": tech, evidence

#tech(
  id: "4.22.4",
  title-zh: "携物梯子",
  title-en: "Holdable Laddering",
  status: "unimplemented",
  description-zh: [交替放下并抓取两只水母，可反复保留纵向或水平速度并持续上升；Theo 版本需要下方移动平台托举。],
  description-en: [Alternating two jelly drops and regrabs can sustain upward movement, while Theo laddering needs a rising support below.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Source/Holdable.cs / Source/Glider.cs / Source/TheoCrystal.cs],
    symbol: [Player.Drop / Player.PickupCoroutine / Holdable.Release / Glider.OnRelease / TheoCrystal.OnRelease],
    snippet: raw(block: true, lang: "cs", "Holding.Release(Vector2.Zero);\n...\ncannotHoldTimer = cannotHoldDelay;\n...\nSpeed = oldSpeed;\nSpeed.Y = Math.Min(Speed.Y, 0);"),
    note: [梯子依赖交替两只实体的独立 CannotHold、位置、速度和 Pickup tween；Glider 使用 0.3 秒 CannotHold。Rust 快照现保留逐实体 Glider 状态与 Holding 索引，并验证第一只锁定时第二只仍可抓取；持续上升的真实链仍需真机证明。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs / crates/celeste-physics/src/sim.rs], symbol: [GliderSnapshot / gliders / holding_glider / release_glider]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [two_gliders_keep_independent_laddering_lockouts]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.22.4-holdable-laddering.ts], symbol: [entity-4.22.4-holdable-laddering], note: [独立 MapPart 放置两只 Jelly 并尝试三次交替 Pickup；真实持续上升与视频尚待 FIFO 锁内验收。]),
)
