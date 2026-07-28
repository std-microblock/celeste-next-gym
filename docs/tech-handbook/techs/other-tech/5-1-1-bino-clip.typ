#import "../../template.typ": tech, evidence

#tech(
  id: "5.1.1",
  title-zh: "望远镜卸载穿越",
  title-en: "Bino Clip",
  status: "unimplemented",
  description-zh: [观察远处时屏幕外 Spinner 会被卸载；快速退出并在其重新加载前移动，可暂时安全穿过原本致死区域。],
  description-en: [Moving the binocular camera away makes off-screen spinners invisible and non-collidable; after exiting, their interval-gated view check leaves a brief crossing window. Rust now links spinner lifecycle to the live Lookout camera; real clip certification is pending.],
  source-evidence: evidence(
    path: [Celeste/Lookout.cs; Celeste/CrystalStaticSpinner.cs],
    symbol: [Lookout.LookRoutine; CrystalStaticSpinner.Update; CrystalStaticSpinner.InView],
    snippet: raw(block: true, lang: "cs", "if (!Visible) {\n    Collidable = false;\n    if (InView()) Visible = true;\n} else {\n    if (Scene.OnInterval(0.25f, offset) && !InView())\n        Visible = false;\n}\n...\nreturn X > camera.X - 16f && X < camera.X + 336f;"),
    note: [Lookout 直接移动 `Level.Camera.Position`；Spinner 每次不可见时先关闭 Collidable，而离开视野只在带实体 offset 的 0.25 秒 interval 上发生，因此退出望远镜后可短暂经过尚未重新启用的危险区。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/types.rs], symbol: [spinner_in_view; advance_spinners; advance_free_lookout_camera; SpinnerSnapshot], note: [Spinner `InView` 读取实时 `PlayerSnapshot.camera`，保持带 offset 的 0.25/0.05 秒 interval；不可见分支先清 Collidable，再按镜头重载。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [bino_clip_uses_live_camera_and_spinner_interval_state], note: [回归先观察 spinner 可见，再将 Lookout 镜头移出 500px，验证 interval 后 Visible/Collidable 同时关闭。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.1.1-bino-clip.ts; scripts/e2e-real/scenarios/lookout-parts.ts], symbol: [other-5.1.1-bino-clip; TECH_OTHER_5_1_1_BINO_CLIP], note: [独立 Lookout+Spinner MapPart 从 Everest 读取 camera、Visible 与 Collidable；真实穿越窗口尚待 semantic gate。]),
)
