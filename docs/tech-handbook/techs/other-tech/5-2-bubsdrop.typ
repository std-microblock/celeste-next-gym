#import "../../template.typ": tech, evidence

#tech(
  id: "5.2",
  title-zh: "Bubsdrop 回房下落",
  title-en: "Bubsdrop",
  status: "implemented",
  description-zh: [在向上房间切换时用墙跳或攀跳取消上升动量，避免落上单向平台并掉回旧房间，从而触发新的出生点选择。],
  description-en: [An upward transition installs a -105 auto-jump; a wallkick or climb jump changes the post-transition path so the player misses a jumpthrough, falls back, and makes the old room choose a different nearest spawn. Rust preserves every room runtime (including the source room) and its full spawn set, so the return transfer restores old-room collision before choosing that room's nearest spawn.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs; Celeste/JumpThru.cs; Celeste/Level.cs],
    symbol: [Player.BeforeUpTransition; Player.NormalUpdate; Player.Update; JumpThru.JumpThru; Level.TransitionRoutine],
    snippet: raw(block: true, lang: "cs", "public void BeforeUpTransition() {\n    Speed.X = 0;\n    varJumpSpeed = Speed.Y = JumpSpeed;\n    StateMachine.State = StNormal;\n    AutoJump = true;\n}\n...\nif (!onGround && Speed.Y <= 0 && CollideCheck<JumpThru>())\n    MoveV(JumpThruAssistSpeed * Engine.DeltaTime); // -40\n// JumpThru collider: new Hitbox(width, 5f, 0f, 0f)\nSession.RespawnPoint = Session.LevelData.Spawns.ClosestTo(player.Position);"),
    note: [上切房把玩家改为 Normal 并强制 `Speed.Y=-105` 与 AutoJump；墙跳后的下一帧，Player.Update 会在常规 Actor 垂直位移之前，以 `-40 px/s` 通过五像素高的 JumpThru collider 上推。切房结束后才以最终位置选最近出生点。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/map.rs], symbol: [begin_transition; update_transition; RoomRuntime; room_spawns], note: [解码会保存每一个房间（含初始 source room）的 entities、solids 与完整 spawn 列表；transition completion 先替换目标房 runtime，再按平方距离从该房 spawn 集选择 RespawnPoint，Session 级状态不随之重置。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/map.rs], symbol: [bubsdrop_wall_jump_misses_upper_jumpthru_and_restores_old_room_spawn_set; touching_jump_thru; selected_room_retains_adjacent_transition_bounds], note: [Rust 回归对照无输入自动跳落上 JumpThru 与第 41 帧墙跳：后者以 `(-130,-105)` 离开平台、回到旧房，并在出界死亡后重生于旧房两点 spawn 中距离最近的 `(24,32)`。它逐帧锁定 f42–f45 的位置、速度和 movement remainder，包含 f44 的 `JumpThruAssistSpeed=-40` 后再执行 `-105` Actor MoveV；二进制解码回归确认初始 source room 也被保存在 runtime 集合。]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.2-bubsdrop.ts; scripts/e2e-real/scenarios/bubs-parts.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs; crates/celeste-physics/src/sim.rs], symbol: [other-5.2-bubsdrop; TECH_OTHER_5_2_BUBSDROP; sessionRespawnPoint; bubsdrop_wall_jump_misses_upper_jumpthru_and_restores_old_room_spawn_set; touching_jump_thru], note: [2026-07-28 在仓库物理 `vendor/celeste-game` 的隔离 Everest run 上执行。动态端口 53547/53548、run nonce 与本次 spawned Celeste PID 105296 已在 per-run manifest 认证，save/tmp 均隔离；cleanup 只终止该 manifest 所有的 game/service 子进程。141 个状态逐帧比较 position、speed、state、facing、dashes、stamina、grounded、ducking、death，最大 position 误差 0.000019、speed 误差 0.000047，均不超过 0.01。真实 trace 在 f42/f43 为 `(450,-7)`/`(448,-9)` 与 `(-130,-105)`；f44 先执行 -40 JumpThru assist、再执行 -105 Actor MoveV，得到 `(446,-11)`、`(-119.16665,-105)`。f114 起 collector 的 Session.RespawnPoint 是旧房较近 spawn `[440,496]`。]),
  candidate-e2e: none,
)
