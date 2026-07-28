#import "../../template.typ": tech, evidence

#tech(
  id: "5.2",
  title-zh: "Bubsdrop 回房下落",
  title-en: "Bubsdrop",
  status: "unimplemented",
  description-zh: [在向上房间切换时用墙跳或攀跳取消上升动量，避免落上单向平台并掉回旧房间，从而触发新的出生点选择。],
  description-en: [An upward transition installs a -105 auto-jump; a wallkick or climb jump changes the post-transition path so the player misses a jumpthrough, falls back, and makes the old room choose a different nearest spawn. Rust preserves every room runtime (including the source room) and its full spawn set, so the return transfer restores old-room collision before choosing that room's nearest spawn; real Everest confirmation is still pending.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste/Level.cs],
    symbol: [Player.BeforeUpTransition; Player.NormalUpdate; Level.TransitionRoutine],
    snippet: raw(block: true, lang: "cs", "public void BeforeUpTransition() {\n    Speed.X = 0;\n    varJumpSpeed = Speed.Y = JumpSpeed;\n    StateMachine.State = StNormal;\n    AutoJump = true;\n    varJumpTimer = VarJumpTime;\n}\n...\nSession.RespawnPoint = Session.LevelData.Spawns.ClosestTo(player.Position);"),
    note: [上切房把玩家改为 Normal 并强制 `Speed.Y=-105` 与 AutoJump；切房结束后才以最终位置选最近出生点。墙跳/攀跳发生在两次切房之间，改变是否落上 JumpThru，从而决定回房位置和 `RespawnPoint`。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/map.rs], symbol: [begin_transition; update_transition; RoomRuntime; room_spawns], note: [解码会保存每一个房间（含初始 source room）的 entities、solids 与完整 spawn 列表；transition completion 先替换目标房 runtime，再按平方距离从该房 spawn 集选择 RespawnPoint，Session 级状态不随之重置。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/map.rs], symbol: [bubsdrop_wall_jump_misses_upper_jumpthru_and_restores_old_room_spawn_set; selected_room_retains_adjacent_transition_bounds], note: [Rust 回归对照无输入自动跳落上 JumpThru 与第 41 帧墙跳：后者以 `(-130,-105)` 离开平台、回到旧房，并在出界死亡后重生于旧房两点 spawn 中距离最近的 `(24,32)`；二进制解码回归确认初始 source room 也被保存在 runtime 集合。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.2-bubsdrop.ts; scripts/e2e-real/scenarios/bubs-parts.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs; crates/celeste-physics/src/sim.rs], symbol: [other-5.2-bubsdrop; TECH_OTHER_5_2_BUBSDROP; sessionRespawnPoint; bubsdrop_wall_jump_misses_upper_jumpthru_and_restores_old_room_spawn_set], note: [独立双房 MapPart 在上房放置相邻 wall 与 JumpThru，并在旧房注入第二个 `(440,496)` spawn；脚本验证 -105 auto-jump、`(-130,-105)` 墙跳、y>0 回旧房以及 collector 的 `[440,496]` Session.RespawnPoint。真实 trace 的首个墙跳边界为 state 42 `(-130,-105)`、state 43 保持 `(-130,-105)`、state 44 才按空气摩擦到 `(-119.16665,-105)`；Rust 回归逐帧锁定该顺序。等待修复后的主工作区真实 Everest trace 后转正。]),
)
