#import "../../template.typ": tech, evidence

#tech(
  id: "3.10",
  title-zh: "尖刺跳",
  title-en: "Spike Jumps",
  status: "implemented",
  description-zh: [风或移动实体可在碰撞检查之后把玩家与尖刺相对推动，使玩家从通常致死的尖刺表面获得合法跳跃帧。],
  description-en: [Wind or moving solids can shift the player relative to spikes after collision checks, creating frames where a jump from a spike surface is legal.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste.ZipMover.Sequence],
    symbol: [Player.Update; ZipMover.Sequence; Solid.MoveTo],
    snippet: raw(block: true, lang: "cs", "MoveH(Speed.X * Engine.DeltaTime, onCollideH);\nMoveV(Speed.Y * Engine.DeltaTime, onCollideV);\n...\nforeach (PlayerCollider component in Scene.Tracker.GetComponents<PlayerCollider>())\n    component.Check(this);\n...\nMoveTo(Vector2.Lerp(start, target, percent));"),
    note: [玩家自身移动和 PlayerCollider 尖刺检查先完成；ZipMover 的协程随后以 Solid.MoveTo 携带 rider，形成一个未再跑尖刺 PlayerCollider 的落脚状态，下一玩家帧即可正常跳起。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [apply_zip_movers; spike_is_lethal]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [spike_jump_uses_the_frame_after_zip_carry_bypasses_player_colliders]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/spike-jump.ts], symbol: [spike-jump; verifySpikeJump], note: [独立 ZipMover 与上刺 MapPart 共 36 个真实状态；state 27 是携带后的合法接地帧，state 28 以 Y=-105 离地且全程存活。九类字段逐帧一致，最大位置误差 0、速度误差 0.000001。]),
  candidate-e2e: none,
)
