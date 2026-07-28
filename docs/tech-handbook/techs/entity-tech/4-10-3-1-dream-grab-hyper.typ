#import "../../template.typ": tech, evidence

#tech(
  id: "4.10.3.1",
  title-zh: "Dream Grab Hyper",
  title-en: "Dream Grab Hyper",
  status: "implemented",
  description-zh: [Dream Grab 不会清除出口土狼时间，因此抓住梦块外墙后仍能接 Hyper。],
  description-en: [Dream grab does not clear exit coyote time, so a hyper remains available after catching the dream-block wall.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.DreamDashUpdate / Player.DreamDashEnd / Player.SuperJump],
    snippet: raw(block: true, lang: "cs", "if (Input.Grab.Check && ClimbCheck(dir))\n    return StClimb;\n...\nif (DashDir.X != 0f)\n    jumpGraceTimer = 0.1f;"),
    note: [DreamDashUpdate 可在横向出口同帧返回 Climb；状态切换后的 DreamDashEnd 仍为水平出口写入 0.1 秒 jump grace。松开抓墙并输入蹲冲刺跳时，这个 grace 允许 SuperJump 走 Hyper 分支。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [interact / climb_update / super_jump], note: [出口先执行五像素修正与两侧 ClimbCheck，再恢复 jump grace、dash、stamina；后续输入按 Climb→Normal→Hyper 的源顺序消费窗口。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [dream_grab_catches_the_block_wall_on_the_exit_frame / hyperdash_applies_duck_super_multipliers]),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [entity-4.10.3.1-dream-grab-hyper],
    note: [真实独立 MapPart 共 76 个状态，完整命中 DreamDash 出口抓墙与随后 Hyper；position/speed 最大误差均为 0，state、facing、dashes、stamina、grounded、ducking、death 在修正 dash-jump facing 顺序后全部逐帧一致。],
  ),
  candidate-e2e: none,
)
