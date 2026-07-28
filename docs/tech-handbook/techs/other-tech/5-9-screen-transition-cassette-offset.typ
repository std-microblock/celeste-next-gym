#import "../../template.typ": tech, evidence

#tech(
  id: "5.9",
  title-zh: "房间切换卡带偏移",
  title-en: "Screen Transition Cassette Offset",
  status: "implemented",
  description-zh: [卡带 beat 切色与跨房实体加载同帧发生时，上一颜色和当前颜色 CassetteBlock 的启停位移会落在不同更新阶段，造成相反的一像素偏移。Rust 现在 TransitionRoutine 启动时装载目标房，global manager 保持 beat；目标 block 静默按旧 index 初始化，随后同帧 WillToggle 使两色进入相反的 1px 阶段。],
  description-en: [When a cassette beat changes color on the same frame as room loading, old- and new-room blocks enter different one-pixel phases. Rust now loads the destination room when TransitionRoutine starts, silently initializes its blocks from the retained global index, and then lets that frame's WillToggle put the two colors in opposite one-pixel phases.],
  source-evidence: evidence(
    path: [Celeste/Level.cs; Celeste/CassetteBlockManager.cs; Celeste/CassetteBlock.cs],
    symbol: [Level.TransitionRoutine; CassetteBlockManager.OnLevelStart; CassetteBlock.SetActivatedSilently],
    snippet: raw(block: true, lang: "cs", "Session.Level = next.Name;\nLoadLevel(Player.IntroTypes.Transition);\n...\nentity.SetActivatedSilently(entity.Index == currentIndex);\n...\npublic void WillToggle() { ShiftSize(Collidable ? 1 : -1); }"),
    note: [TransitionRoutine 在过场开头载入新房；Global manager 保留旧 beat，再由 OnLevelStart 只静默初始化新房 block。随后本帧 manager 的 WillToggle 对按旧 index 初始化为 active 的 block 与 inactive block 分别下移、上移 1px，形成偏移。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/map.rs], symbol: [begin_transition; load_transition_room; initialize_cassette_blocks; advance_cassette_manager; RoomRuntime], note: [begin_transition 先以目标房替换 room-local entities，并保留 global manager 的 `current_index`；initialize_cassette_blocks 只执行 SetActivatedSilently 等效初始化。紧随边界检查之后的 entity 更新先执行 block，再由 manager 的 beat 7 WillToggle 对两个相反状态分别 ShiftSize ±1。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [transition_loads_destination_cassettes_before_same_frame_will_toggle; cassette_manager_keeps_advancing_during_room_transition], note: [回归将 beat 精确设在 6→7，断言进入 transition 后目标房 index-0 block 位于 source Y+2、按旧 index 初始化的目标 index-1 位于 source Y；同一帧结束后两者均在 source Y+1，分别来自上移与下移 1px。另一回归锁定 transition 内 manager 仍推进 beat。]),
  e2e-evidence: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.9-screen-transition-cassette-offset.ts; scripts/e2e-real/scenarios/cassette-spinner-parts.ts; mods/CelesteGymCollector/Source/SnapshotCapture.cs], symbol: [other-5.9-screen-transition-cassette-offset; TECH_OTHER_5_9_TRANSITION_CASSETTE_OFFSET; SnapshotCapture.Capture], note: [2026-07-28 隔离真实 Everest run `2026-07-28T13-46-22.679Z-99676-d254cd3f-e81b-4506-9f90-9a6df3c44304` 完成 nonce/PID 认证和受控清理；91 帧九字段逐帧匹配（最大 position 误差 0.000019、speed 误差 0）。transition 中 collector 同帧看到四个 block：旧／新 index-0 分别为 Y=18/-46 且非碰撞，index-1 分别为 Y=16/-48 且可碰撞；这证明两房载入期间保持各自 cassette 相位和 offset。]),
  candidate-e2e: none,
)
