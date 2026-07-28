#import "../../template.typ": tech, evidence

#tech(
  id: "5.1.1",
  title-zh: "望远镜卸载穿越",
  title-en: "Bino Clip",
  status: "implemented",
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
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.1.1-bino-clip.ts; scripts/e2e-real/scenarios/lookout-parts.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs; .tmp/e2e-runs/2026-07-28T20-19-44.372Z-115676-51383c6f-aad4-45cb-949e-a207201adc6b/manifest.json], symbol: [other-5.1.1-bino-clip; TECH_OTHER_5_1_1_BINO_CLIP; SnapshotCapture.Capture], note: [2026-07-28 在仓库物理 `vendor/celeste-game` 的隔离 Everest run 上完成。动态端口 53326/53327、run nonce 与 spawned Celeste PID 113732 已认证，save/tmp 隔离且 cleanup 仅终止受控子进程。231 个状态逐帧比较 position、speed、state、facing、dashes、stamina、grounded、ducking、death，position/speed 最大误差均为 0。真实 trace 在 f89 的 camera x 为 249.99988；f90 CrystalStaticSpinner 已 Visible=false 而仍 Collidable=true，f91 由下一实体 update 变为 Visible=false、Collidable=false，验证离屏 0.25 秒 interval 及下一帧碰撞关闭。]),
  candidate-e2e: none,
)
