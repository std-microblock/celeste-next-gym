#import "../../template.typ": tech, evidence

#tech(
  id: "5.10",
  title-zh: "Spinner 暂停眩晕",
  title-en: "Spinner Stunning",
  status: "unimplemented",
  description-zh: [CrystalStaticSpinner 的 0.05 秒 offset 检查与距离碰撞模型已实现；技巧还要求在命中帧调用游戏外 Pause，使 Scene/实体 Update 被跳过。按用户口径，暂停控制只做机制解析，不在当前 runner 伪造，因此 verdict 保持未实现。],
  description-en: [The 0.05-second offset/proximity spinner runtime is implemented. The technique additionally requires an out-of-game Pause on the service frame; by product decision that control is mechanism-only and is not faked by the runner, so the verdict remains unimplemented.],
  source-evidence: evidence(
    path: [Celeste/CrystalStaticSpinner.cs; Monocle/Scene.cs],
    symbol: [CrystalStaticSpinner.Update; Scene.BeforeUpdate; Scene.Update],
    snippet: raw(block: true, lang: "cs", "Visible = false;\n...\nif (!Visible) {\n    Collidable = false;\n    if (InView()) Visible = true;\n}\n...\nif (Scene.OnInterval(0.05f, offset))\n    Collidable = Math.Abs(player.X - X) < 128f && Math.Abs(player.Y - Y) < 128f;\n...\nif (!Paused) Entities.Update();"),
    note: [构造器只把 Visible 设为 false，Entity.Collidable 的默认 true 会保留到 Spinner 第一次 Update；随后只有命中该 spinner offset 的 interval 才刷新 proximity Collidable。Pause 阻止实体 Update，所以被跳过的命中不会补执行。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/types.rs], symbol: [scene_on_interval; advance_spinners; SpinnerSnapshot], note: [Rust 保留 per-spinner offset、Visible/Collidable 与 128px proximity，并在 Player callback 后更新；没有 Pause 输入或菜单状态。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [fresh_invisible_spinner_keeps_constructor_collision_for_player_phase; spinner_proximity_check_enables_collision_after_player_callback_phase], note: [回归分别锁定 fresh invisible Spinner 在第一次 Player callback 仍可碰撞，以及后续 interval 命中帧只把 Collidable 打开、玩家到下一 callback 才死亡；暂停窗口本身按产品决定不实现。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.10-spinner-stunning.ts; scripts/e2e-real/scenarios/cassette-spinner-parts.ts], symbol: [other-5.10-spinner-stunning; TECH_OTHER_5_10_SPINNER_STUNNING], note: [独立 Spinner baseline 的首次真实运行在 frame 1 得到 Everest dead=true、Rust dead=false，暴露了 fresh invisible Spinner 的构造期 Collidable 差异；runtime 与回归已按源码修正，尚待一次真实 baseline 复核。即使 baseline 通过，runner 仍无 Pause 控制，不能把它当作 Stunning 证据。]),
)
