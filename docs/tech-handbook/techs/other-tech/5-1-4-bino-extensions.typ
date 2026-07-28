#import "../../template.typ": tech, evidence

#tech(
  id: "5.1.4",
  title-zh: "望远镜扩展",
  title-en: "Bino Extensions",
  status: "unimplemented",
  description-zh: [改变望远镜记录的中心或退出位置，可以把镜头移动到常规范围之外，从而扩大卸载与穿越技巧的作用距离。],
  description-en: [Lookout caches the initial camera position and center, then uses them for node interpolation and the long-distance exit wipe. Rust now models the node curves and wipe; real extension certification is pending.],
  source-evidence: evidence(
    path: [Celeste/Lookout.cs],
    symbol: [Lookout.LookRoutine],
    snippet: raw(block: true, lang: "cs", "Vector2 camStart = level.Camera.Position;\nVector2 camStartCenter = camStart + new Vector2(160f, 90f);\n...\nVector2 from = node <= 0 ? camStartCenter : nodes[node - 1];\nlevel.Camera.Position = Vector2.Lerp(from, nodes[node], nodePercent);\n...\nlevel.Camera.Position = camStart + direction * 32f;"),
    note: [`camStart` 与 `camStartCenter` 只在进入观察阶段时缓存；节点轨迹首段和超过 600 像素后的退出落点都复用这些值，因此更改缓存中心/出口会把相机范围向常规边界之外延伸。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/map.rs], symbol: [LookoutSnapshot; advance_node_lookout_camera; quadratic_curve; EntityKind.Lookout], note: [地图保留节点与 summit/onlyY；快照保存 camStart/nodePercent，0.25/0.75 接缝走二次 SimpleCurve，超过 600px 后按 0.5/1 秒 CubeIn wipe 落到 `camStart + direction*32`。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [bino_extensions_follow_nodes_and_run_long_distance_exit_wipe], note: [回归沿 800px summit node 前进，观察 phase 6，并在 wipe 完成帧验证相机落点 32px。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.1.4-bino-extensions.ts; crates/celeste-physics/src/sim.rs], symbol: [other-5.1.4-bino-extensions; advance_node_lookout_camera; bino_extensions_follow_nodes_and_run_long_distance_exit_wipe], note: [2026-07-29 的 Everest trace 在 frame 453 到达 node 2、nodePercent=1 后仍保持 Dummy/interacting；summit endpoint 只钳制节点，不自动退出。对 `f65fc1b` 的重跑在 frame 460 注入 portable jump 后至 frame 720 仍为 Dummy/interacting，未开始 long-distance wipe，语义门失败；保持 candidate，待核对 LookRoutine 的实际退出绑定后重跑。]),
)
