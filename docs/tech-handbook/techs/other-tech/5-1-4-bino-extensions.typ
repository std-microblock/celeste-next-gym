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
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.1.4-bino-extensions.ts; scripts/e2e-real/scenarios/lookout-parts.ts; mods/CelesteGymCollector/Source/CelesteGymCollectorModule.cs], symbol: [other-5.1.4-bino-extensions; TECH_OTHER_5_1_4_BINO_EXTENSIONS; InstallScriptedButtons], note: [旧 trace `.tmp/e2e-other-5.1.4-bino-extensions-trace.json` 在 f500 已抵达末节点 `node=2, percent=1`，至 f720 仍为 `Dummy + interacting`，证明终点不是自动退出。候选改为 f520 以 jump 映射 `MenuCancel`，再保留 summit 的一秒 wipe 等待。候选 SHA `a5f765701cbc942fd1abe810db7e35f1e9a9150f` 的受控 Everest run `2026-07-28T19-41-39.447Z-114340-95f51b7f-1cd4-499c-a6a1-c1840a1a62db` 将 trace 写入 `.tmp/e2e-other-5.1.4-bino-extensions-trace.json`，comparator 在 f2 即失败（游戏 `Dummy`、Rust `Normal`；最大 position=7、speed=105.000420）；manifest 记录 runner 已完成受控 cleanup。故真实 E2E 尚未通过，保持 candidate。]),
)
