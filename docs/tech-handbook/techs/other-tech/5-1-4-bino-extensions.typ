#import "../../template.typ": tech, evidence

#tech(
  id: "5.1.4",
  title-zh: "望远镜扩展",
  title-en: "Bino Extensions",
  status: "unimplemented",
  description-zh: [改变望远镜记录的中心或退出位置，可以把镜头移动到常规范围之外，从而扩大卸载与穿越技巧的作用距离。],
  description-en: [Lookout caches the initial camera position and center, then uses them for node interpolation and the long-distance exit wipe. Manipulating those stored values extends camera travel, but Rust has no Lookout camera state or nodes, so extensions remain unimplemented.],
  source-evidence: evidence(
    path: [Celeste/Lookout.cs],
    symbol: [Lookout.LookRoutine],
    snippet: raw(block: true, lang: "cs", "Vector2 camStart = level.Camera.Position;\nVector2 camStartCenter = camStart + new Vector2(160f, 90f);\n...\nVector2 from = node <= 0 ? camStartCenter : nodes[node - 1];\nlevel.Camera.Position = Vector2.Lerp(from, nodes[node], nodePercent);\n...\nlevel.Camera.Position = camStart + direction * 32f;"),
    note: [`camStart` 与 `camStartCenter` 只在进入观察阶段时缓存；节点轨迹首段和超过 600 像素后的退出落点都复用这些值，因此更改缓存中心/出口会把相机范围向常规边界之外延伸。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/map.rs], symbol: [PlayerSnapshot; EntityKind], note: [快照没有 Camera、camStart、node/nodePercent，地图实体也没有 Lookout nodes；现有 Player 坐标与 room bounds 无法代替相机轨迹和退出 wipe。]),
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
