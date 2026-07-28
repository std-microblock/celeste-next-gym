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
    snippet: raw(block: true, lang: "cs", "if (Scene.OnInterval(0.05f, offset))\n    Collidable = Math.Abs(player.X - X) < 128f && Math.Abs(player.Y - Y) < 128f;\n...\nif (!Paused) TimeActive += Engine.DeltaTime;\nif (!Paused) Entities.Update();"),
    note: [只有命中该 spinner offset 的 interval 才刷新 Collidable；Pause 同时阻止 TimeActive 与 Entities.Update，所以被跳过的命中不会补执行。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/types.rs], symbol: [scene_on_interval; advance_spinners; SpinnerSnapshot], note: [Rust 保留 per-spinner offset、Visible/Collidable 与 128px proximity，并在 Player callback 后更新；没有 Pause 输入或菜单状态。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [spinner_proximity_check_enables_collision_after_player_callback_phase], note: [回归验证 interval 命中帧只把 Collidable 打开，玩家要到下一实体 callback 帧才死亡；暂停窗口本身按产品决定不实现。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.10-spinner-stunning.ts; scripts/e2e-real/scenarios/cassette-spinner-parts.ts], symbol: [other-5.10-spinner-stunning; TECH_OTHER_5_10_SPINNER_STUNNING], note: [独立 Spinner MapPart 只验证未暂停 baseline 会启用碰撞；runner 无 Pause 控制，不能把 baseline 当作 Stunning 证据。]),
)
