#import "../../template.typ": tech, evidence

#tech(
  id: "5.2",
  title-zh: "Bubsdrop 回房下落",
  title-en: "Bubsdrop",
  status: "unimplemented",
  description-zh: [在向上房间切换时用墙跳或攀跳取消上升动量，避免落上单向平台并掉回旧房间，从而触发新的出生点选择。],
  description-en: [An upward transition installs a -105 auto-jump; a wallkick or climb jump changes the post-transition path so the player misses a jumpthrough, falls back, and makes the old room choose a different nearest spawn. Rust now retains room-local spawn sets and replaces them at transition completion; real Bubsdrop timing is still pending.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste/Level.cs],
    symbol: [Player.BeforeUpTransition; Player.NormalUpdate; Level.TransitionRoutine],
    snippet: raw(block: true, lang: "cs", "public void BeforeUpTransition() {\n    Speed.X = 0;\n    varJumpSpeed = Speed.Y = JumpSpeed;\n    StateMachine.State = StNormal;\n    AutoJump = true;\n    varJumpTimer = VarJumpTime;\n}\n...\nSession.RespawnPoint = Session.LevelData.Spawns.ClosestTo(player.Position);"),
    note: [上切房把玩家改为 Normal 并强制 `Speed.Y=-105` 与 AutoJump；切房结束后才以最终位置选最近出生点。墙跳/攀跳发生在两次切房之间，改变是否落上 JumpThru，从而决定回房位置和 `RespawnPoint`。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/map.rs], symbol: [begin_transition; update_transition; RoomRuntime], note: [每个解码房间保留 entities、solids 与 spawn 列表；transition completion 按平方距离选目标房最近 spawn，替换 room-local collision/runtime，保留跨房 Session 级状态。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [transition_loads_destination_cassettes_and_nearest_room_spawn], note: [回归以双 spawn 目标房验证 transition 选择靠近玩家的 spawn，同时不留下旧房 cassette 实体。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.2-bubsdrop.ts; scripts/e2e-real/scenarios/bubs-parts.ts], symbol: [other-5.2-bubsdrop; TECH_OTHER_5_2_BUBSDROP], note: [独立双房/JumpThru/墙面 MapPart 先验证 BeforeUpTransition 的 -105 auto-jump；真实墙跳回房与 RespawnPoint 选择未闭环，保持 candidate。]),
)
