#import "../../template.typ": tech, evidence

#tech(
  id: "5.1.4",
  title-zh: "望远镜扩展",
  title-en: "Bino Extensions",
  status: "implemented",
  description-zh: [改变望远镜记录的中心或退出位置，可以把镜头移动到常规范围之外，从而扩大卸载与穿越技巧的作用距离。],
  description-en: [Lookout caches the initial camera position and center, then uses them for node interpolation and the long-distance exit wipe. The certified scenario reaches the summit endpoint, explicitly sends MenuCancel, and completes the long-distance wipe.],
  source-evidence: evidence(
    path: [Celeste/Lookout.cs],
    symbol: [Lookout.LookRoutine],
    snippet: raw(block: true, lang: "cs", "Vector2 camStart = level.Camera.Position;\nVector2 camStartCenter = camStart + new Vector2(160f, 90f);\n...\nVector2 from = node <= 0 ? camStartCenter : nodes[node - 1];\nlevel.Camera.Position = Vector2.Lerp(from, nodes[node], nodePercent);\n...\nlevel.Camera.Position = camStart + direction * 32f;"),
    note: [`camStart` 与 `camStartCenter` 只在进入观察阶段时缓存；节点轨迹首段和超过 600 像素后的退出落点都复用这些值，因此更改缓存中心/出口会把相机范围向常规边界之外延伸。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/map.rs], symbol: [LookoutSnapshot; advance_node_lookout_camera; quadratic_curve; EntityKind.Lookout], note: [地图保留节点与 summit/onlyY；快照保存 camStart/nodePercent，0.25/0.75 接缝走二次 SimpleCurve，超过 600px 后按 0.5/1 秒 CubeIn wipe 落到 `camStart + direction*32`。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [bino_extensions_follow_nodes_and_run_long_distance_exit_wipe], note: [回归沿 800px summit node 前进，观察 phase 6，并在 wipe 完成帧验证相机落点 32px。]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.1.4-bino-extensions.ts; scripts/e2e-real/scenarios/lookout-parts.ts; mods/CelesteGymCollector/Source/CelesteGymCollectorModule.cs; .tmp/e2e-runs/2026-07-28T19-56-34.333Z-115364-4693aa59-bb4c-4b55-b4d4-88c043fca5f4/manifest.json], symbol: [other-5.1.4-bino-extensions; TECH_OTHER_5_1_4_BINO_EXTENSIONS; InstallScriptedButtons], note: [候选 SHA `a9475436c50bd3e7ada48190e0a233b56265f0bf` 的 2026-07-28 隔离真实 Everest run 在物理 `vendor/celeste-game` 上执行；per-run manifest 记录隔离 save/tmp、动态端口 50310/50311、nonce `548a8908-1150-4c37-b6dc-573749c52c02` 与本次 spawned Celeste PID `74652` 的精确握手，以及受控 cleanup。641 个状态的 position、speed、state、facing、dashes、stamina、grounded、ducking、death 全部逐帧匹配，position/speed 最大误差均为 0。f519 为 node=2、percent=1 且仍 interacting；f520 的 raw jump 映射 `MenuCancel`，f521--f580 保持 Dummy/interacting 完成 summit 一秒 wipe，f581 恢复 Normal/non-interacting；相机位移超过 600px。]),
  candidate-e2e: none,
)
