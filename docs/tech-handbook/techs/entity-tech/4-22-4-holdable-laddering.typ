#import "../../template.typ": tech, evidence

#tech(
  id: "4.22.4",
  title-zh: "携物梯子",
  title-en: "Holdable Laddering",
  status: "implemented",
  description-zh: [交替放下并抓取两只水母，可反复保留纵向或水平速度并持续上升；Theo 版本需要下方移动平台托举。],
  description-en: [Alternating two jelly drops and regrabs can sustain upward movement, while Theo laddering needs a rising support below.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Source/Holdable.cs / Source/Glider.cs / Source/TheoCrystal.cs],
    symbol: [Player.Drop / Player.PickupCoroutine / Holdable.Release / Glider.OnRelease / TheoCrystal.OnRelease],
    snippet: raw(block: true, lang: "cs", "Holding.Release(Vector2.Zero);\n...\ncannotHoldTimer = cannotHoldDelay;\n...\nSpeed = oldSpeed;\nSpeed.Y = Math.Min(Speed.Y, 0);"),
    note: [梯子依赖交替两只实体的独立 CannotHold、位置、速度和 Pickup tween；`Glider` 源码为 0.3 秒 CannotHold 和 20×22 PickupCollider。Rust 快照保留逐实体 Glider 状态与 Holding 索引，并验证第一只锁定时第二只仍可抓取；真实复验已确认 20×22 拾取框消除了旧的第 25 帧 Pickup 差异，但后续仍有重力／速度时序偏差，故持续上升链仍需真机证明。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs / crates/celeste-physics/src/sim.rs], symbol: [GliderSnapshot / gliders / holding_glider / release_glider]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [two_gliders_keep_independent_laddering_lockouts]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.22.4-holdable-laddering.ts; .tmp/e2e-runs/2026-07-28T14-54-25.385Z-84784-264b60dd-80d3-4c39-93d0-89bb2994f75d/manifest.json], symbol: [entity-4.22.4-holdable-laddering], note: [2026-07-28 在受锁主工作区的物理 `vendor/celeste-game` 上运行；runner nonce 与 spawned Celeste PID 匹配，隔离存档/临时目录、动态端口及受控清理均完成。151 帧 position、speed、state、facing、dashes、stamina、grounded、ducking、death 逐帧一致，position/speed 最大误差均为 0。]),
)
