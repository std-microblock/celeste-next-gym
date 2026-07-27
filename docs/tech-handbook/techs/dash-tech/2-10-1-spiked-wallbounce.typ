#import "../../template.typ": tech, evidence

#tech(
  id: "2.10.1",
  title-zh: "尖刺墙反",
  title-en: "Spiked Wallbounce",
  status: "implemented",
  description-zh: [利用方向性尖刺和冲刺攻击窗口，在带尖刺墙面上完成墙反；由于常规宽容修正不可用，时序更严格。],
  description-en: [A spiked wallbounce combines directional-spike safety with the dash-attack window and lacks much of a normal wallbounce's leniency.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste.Spikes.OnCollide],
    symbol: [Player.DashUpdate; Player.SuperWallJump; Spikes.OnCollide],
    snippet: raw(block: true, lang: "cs", "if (DashDir.X == 0 && DashDir.Y == -1 && Input.Jump.Pressed) {\n    if (WallJumpCheck(1)) { SuperWallJump(-1); return StNormal; }\n}\n// SuperWallJump: Speed.X = 170 * dir; Speed.Y = -160\n// left-facing spikes only kill while Speed.X >= 0"),
    note: [精确帧的 SuperWallJump 先把水平速度改为远离左刺的 -170，随后同帧 PlayerCollider 尖刺判定因此安全；迟一帧时玩家仍以 X=0 接触左刺并死亡。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [dash_update; super_wall_jump; spike_is_lethal; current_player_hurt_rect]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [spiked_wallbounce_is_safe_on_the_entry_frame_but_dies_one_frame_late]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [dash-spiked-wallbounce; dash-spiked-wallbounce-late], note: [准时输入在 frame 6 得到 -170/-160 且 15 帧全程存活，最大位置误差 0、速度误差 0.000038；迟一帧场景在输入生效前死亡，8 帧九类字段误差 0。]),
  candidate-e2e: none,
)
