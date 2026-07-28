#import "../../template.typ": tech, evidence

#tech(
  id: "4.16",
  title-zh: "岩浆 Neutral",
  title-en: "Lava Neutrals",
  status: "unimplemented",
  description-zh: [岩浆受伤区前有 1 像素实体边缘，可在该像素上缓冲 Neutral、攀跳、墙跳、墙反等墙面动作。],
  description-en: [RisingLava and SandwichLava are camera/core-driven PlayerCollider hazards whose visible edge and lethal collider are separate. Rust has neither entity, camera tracking, core-mode lifecycle, nor the one-pixel safe lip, so Lava Neutrals remain unimplemented rather than approximated with a rectangle.],
  source-evidence: evidence(
    path: [Celeste/RisingLava.cs; Celeste/SandwichLava.cs],
    symbol: [RisingLava.Update / SandwichLava.Update / PlayerCollider],
    snippet: raw(block: true, lang: "cs", "Y = Calc.Approach(Y, level.Camera.Bottom + 10f, 20f * Engine.DeltaTime);\nCollider = new Hitbox(340f, 120f, -10f, 0f);\n...\nPosition.X = level.Camera.X;\nY = Calc.Approach(Y, level.Camera.Y + (iceMode ? -20f : 20f), 20f * Engine.DeltaTime);"),
    note: [RisingLava 用 340×120 PlayerCollider 朝 Camera.Bottom+10 以 20 px/s 追赶；SandwichLava 锁定 camera X，并随 hot/cold core mode 以 ±20 px/s 调整上下危险区，还包含 Waiting、leaving 与跨房间持久状态。技巧依赖视觉边缘与致死 collider 之间的精确 1px 安全唇。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/sim.rs],
    symbol: [EntityKind],
    note: [Rust 尚无 RisingLava/SandwichLava、camera 边界、core hot/cold 生命周期、Waiting/leaving/persistence 与 1px 实体边缘；普通 Solid 或 Spikes 矩形会改变墙面动作与死亡边界，因此不建立近似 MapPart，也不 promotion。],
  ),
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
