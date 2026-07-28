#import "../../template.typ": tech, evidence

#tech(
  id: "5.9",
  title-zh: "房间切换卡带偏移",
  title-en: "Screen Transition Cassette Offset",
  status: "unimplemented",
  description-zh: [卡带 beat 切色与跨房实体加载同帧发生时，上一颜色和当前颜色 CassetteBlock 的启停位移会落在不同更新阶段，造成相反的一像素偏移。Rust 现保留每房 runtime 并在 TransitionRoutine completion 装载目标房，global manager 保持 beat；真实同帧 trace 仍待验收。],
  description-en: [When a cassette beat changes color on the same frame as room loading, old- and new-room blocks enter different one-pixel phases. Rust now loads destination-room runtime while retaining the global manager beat; a real same-frame Everest trace is still required.],
  source-evidence: evidence(
    path: [Celeste/Level.cs; Celeste/CassetteBlockManager.cs; Celeste/CassetteBlock.cs],
    symbol: [Level.TransitionRoutine; CassetteBlockManager.OnLevelStart; CassetteBlock.SetActivatedSilently],
    snippet: raw(block: true, lang: "cs", "Session.Level = next.Name;\nLoadLevel(Player.IntroTypes.Transition);\n...\nentity.SetActivatedSilently(entity.Index == currentIndex);\n...\npublic void WillToggle() { ShiftSize(Collidable ? 1 : -1); }"),
    note: [TransitionRoutine 在过场开头载入新房；Global manager 保留旧 beat，再由 OnLevelStart 只静默初始化新房 block。若这与 WillToggle/SetActiveIndex 同帧，旧房和新房会处在不同的 1px 阶段。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs; crates/celeste-physics/src/map.rs], symbol: [update_transition; initialize_cassette_blocks; RoomRuntime], note: [transition 完成时置换目标房 entities 并用既存 manager index 静默初始化新 cassette；随后的 manager 更新仍在该帧推进 beat，避免把普通跨房 baseline 冒充 offset。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [transition_loads_destination_cassettes_and_nearest_room_spawn; cassette_manager_keeps_advancing_during_room_transition], note: [两条回归分别锁定目标房 cassette 装载和 transition 中 beat 6→7 的 WillToggle；二者都是同一帧序的必要条件。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.9-screen-transition-cassette-offset.ts; scripts/e2e-real/scenarios/cassette-spinner-parts.ts], symbol: [other-5.9-screen-transition-cassette-offset; TECH_OTHER_5_9_TRANSITION_CASSETTE_OFFSET], note: [独立双房/双颜色 MapPart 已通过 BinaryPacker 编译；必须由真实 Everest 同帧 trace 证明新旧房 block offset 后才能晋级。]),
)
