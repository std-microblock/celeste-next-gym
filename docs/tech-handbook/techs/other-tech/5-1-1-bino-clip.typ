#import "../../template.typ": tech, evidence

#tech(
  id: "5.1.1",
  title-zh: "望远镜卸载穿越",
  title-en: "Bino Clip",
  status: "unimplemented",
  description-zh: [观察远处时屏幕外 Spinner 会被卸载；快速退出并在其重新加载前移动，可暂时安全穿过原本致死区域。],
  description-en: [Moving the binocular camera away makes off-screen spinners invisible and non-collidable; after exiting, their interval-gated view check leaves a brief crossing window. Rust has neither binocular camera control nor CrystalStaticSpinner lifecycle, so the clip is not implemented.],
  source-evidence: evidence(
    path: [Celeste/Lookout.cs; Celeste/CrystalStaticSpinner.cs],
    symbol: [Lookout.LookRoutine; CrystalStaticSpinner.Update; CrystalStaticSpinner.InView],
    snippet: raw(block: true, lang: "cs", "if (!Visible) {\n    Collidable = false;\n    if (InView()) Visible = true;\n} else {\n    if (Scene.OnInterval(0.25f, offset) && !InView())\n        Visible = false;\n}\n...\nreturn X > camera.X - 16f && X < camera.X + 336f;"),
    note: [Lookout 直接移动 `Level.Camera.Position`；Spinner 每次不可见时先关闭 Collidable，而离开视野只在带实体 offset 的 0.25 秒 interval 上发生，因此退出望远镜后可短暂经过尚未重新启用的危险区。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/map.rs; crates/celeste-physics/src/sim.rs], symbol: [EntityKind; step], note: [实体模型没有 CrystalStaticSpinner 或 Lookout，模拟快照也没有 Camera、Visible、Collidable 和 `Scene.OnInterval` offset；无法用静态 Spikes 代替这一卸载/重载窗口。]),
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
