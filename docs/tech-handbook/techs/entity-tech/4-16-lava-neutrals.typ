#import "../../template.typ": tech, evidence

#tech(
  id: "4.16",
  title-zh: "岩浆 Neutral",
  title-en: "Lava Neutrals",
  status: "unimplemented",
  description-zh: [岩浆受伤区前有 1 像素实体边缘，可在该像素上缓冲 Neutral、攀跳、墙跳、墙反等墙面动作。],
  description-en: [RisingLava and SandwichLava are camera/core-driven PlayerCollider hazards whose visible edge and lethal collider are separate. Rust has neither entity, camera tracking, core-mode lifecycle, nor the one-pixel safe lip, so Lava Neutrals remain unimplemented rather than approximated with a rectangle.],
  source-evidence: evidence(path: [Celeste/RisingLava.cs; Celeste/SandwichLava.cs], symbol: [RisingLava.Update / SandwichLava.Update / PlayerCollider], note: [两种岩浆都按 camera、core mode 与全局追赶状态更新，并用独立 PlayerCollider 处理死亡；技巧依赖渲染/实体边界前的精确 1px 安全边缘，不能由普通 Solid/Spikes 矩形替代。]),
  rust-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/sim.rs], symbol: [EntityKind], note: [实体枚举与 runtime 没有 RisingLava/SandwichLava、camera 追踪或 core hazard 生命周期。]),
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
