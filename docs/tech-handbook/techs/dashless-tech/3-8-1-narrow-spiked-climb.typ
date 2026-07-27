#import "../../template.typ": tech, evidence

#tech(
  id: "3.8.1",
  title-zh: "窄道尖刺攀爬",
  title-en: "Narrow Spiked Climb",
  status: "implemented",
  description-zh: [在两侧带刺的两格或三格窄道内交替利用尖刺攀爬和墙角跳，以逐段上升。],
  description-en: [A narrow spike climb alternates spike-safe movement and corner jumps inside two- or three-tile spiked shafts.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste.Spikes.OnCollide],
    symbol: [Player.WallJump; Spikes.OnCollide],
    snippet: raw(block: true, lang: "cs", "Speed.X = WallJumpHSpeed * dir;\nSpeed.Y = JumpSpeed;\n...\ncase Directions.Right:\n    if (player.Speed.X <= 0f) player.Die(Vector2.UnitX);\n    break;"),
    note: [每次墙跳都先把速度改成远离当前墙的 +/-130 与 -105；左右尖刺各只惩罚朝其危险方向的速度，故可在窄道中交替安全换墙。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [spike_is_lethal; wall_jump]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [narrow_spiked_climb_alternates_away_facing_wall_jumps]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/narrow-spiked-climb.ts], symbol: [narrow-spiked-climb; verifyNarrowSpikedClimb], note: [独立双墙尖刺 MapPart 共 6 个真实状态；state 1/4 的水平速度分别为 -130/+130，两次换墙均存活并上升。最大位置误差 0、速度误差 0.000015，其余九类字段逐帧一致。]),
  candidate-e2e: none,
)
