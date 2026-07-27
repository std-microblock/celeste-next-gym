#import "../../template.typ": tech, evidence

#tech(
  id: "5.2",
  title-zh: "Bubsdrop 回房下落",
  title-en: "Bubsdrop",
  status: "unimplemented",
  description-zh: [在向上房间切换时用墙跳或攀跳取消上升动量，避免落上单向平台并掉回旧房间，从而触发新的出生点选择。],
  description-en: [An upward transition installs a -105 auto-jump; a wallkick or climb jump changes the post-transition path so the player misses a jumpthrough, falls back, and makes the old room choose a different nearest spawn. Rust models room bounds but not per-room spawn sets or Session.RespawnPoint, so the full technique remains unimplemented.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste/Level.cs],
    symbol: [Player.BeforeUpTransition; Player.NormalUpdate; Level.TransitionRoutine],
    snippet: raw(block: true, lang: "cs", "public void BeforeUpTransition() {\n    Speed.X = 0;\n    varJumpSpeed = Speed.Y = JumpSpeed;\n    StateMachine.State = StNormal;\n    AutoJump = true;\n    varJumpTimer = VarJumpTime;\n}\n...\nSession.RespawnPoint = Session.LevelData.Spawns.ClosestTo(player.Position);"),
    note: [上切房把玩家改为 Normal 并强制 `Speed.Y=-105` 与 AutoJump；切房结束后才以最终位置选最近出生点。墙跳/攀跳发生在两次切房之间，改变是否落上 JumpThru，从而决定回房位置和 `RespawnPoint`。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/map.rs], symbol: [begin_transition; update_transition; step; Map], note: [Rust 已有上下房 bounds 与 BeforeUpTransition 速度，但 `Map` 只保留单一 `spawn`；transition completion 不计算逐房 `Spawns.ClosestTo`，死亡固定回 `map.spawn`，因此技巧的出生点结果无法表达或回归。]),
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
