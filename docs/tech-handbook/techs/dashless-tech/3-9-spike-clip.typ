#import "../../template.typ": tech, evidence

#tech(
  id: "3.9",
  title-zh: "尖刺穿越",
  title-en: "Spike Clip",
  status: "implemented",
  description-zh: [尖刺只检测受伤箱底部像素；速度足够高时可让该像素在相邻帧从尖刺上方直接越到下方而不触发死亡。],
  description-en: [At sufficient speed, the bottom hurtbox pixel can move from above unsupported spikes to below them between frames without intersecting their lethal region.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste.Spikes.OnCollide],
    symbol: [Player.Update; Spikes.OnCollide],
    snippet: raw(block: true, lang: "cs", "MoveH(Speed.X * Engine.DeltaTime, onCollideH);\nMoveV(Speed.Y * Engine.DeltaTime, onCollideV);\n...\ncase Directions.Up:\n    if (player.Speed.Y >= 0f && player.Bottom <= base.Bottom)\n        player.Die(-Vector2.UnitY);"),
    note: [Player 先按整帧速度 MoveH/MoveV，随后 hurtbox PlayerCollider 才调用尖刺。上刺额外要求玩家受伤箱底部仍不低于尖刺底边；一帧跨过这条三像素带后不再致死。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [current_player_hurt_rect; spike_is_lethal]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [spike_clip_requires_the_hurtbox_bottom_to_skip_past_unsupported_spikes]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/spike-clip.ts], symbol: [spike-clip; verifySpikeClip], note: [独立悬空上刺 MapPart 共 7 个真实状态；state 1 的 Y=107、hurtbox bottom 已越过 103 且速度仍大于 220，全程存活。九类字段逐帧一致，位置与速度最大误差均为 0。]),
  candidate-e2e: none,
)
