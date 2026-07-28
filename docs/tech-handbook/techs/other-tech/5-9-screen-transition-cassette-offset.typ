#import "../../template.typ": tech, evidence

#tech(
  id: "5.9",
  title-zh: "房间切换卡带偏移",
  title-en: "Screen Transition Cassette Offset",
  status: "unimplemented",
  description-zh: [卡带 beat 切色与跨房实体加载同帧发生时，上一颜色和当前颜色 CassetteBlock 的启停位移会落在不同更新阶段，造成相反的一像素偏移。Rust 已让 global manager 在 transition 中继续推进，但模拟器仍不会在 TransitionRoutine 中装载下一房实体，因此明确保持未实现。],
  description-en: [When a cassette beat changes color on the same frame as room loading, old- and new-room blocks enter different one-pixel phases. Rust advances the global manager during transitions, but does not load the next room's entities inside TransitionRoutine, so the full technique remains explicitly unimplemented.],
  source-evidence: evidence(
    path: [Celeste/Level.cs; Celeste/CassetteBlockManager.cs; Celeste/CassetteBlock.cs],
    symbol: [Level.TransitionRoutine; CassetteBlockManager.OnLevelStart; CassetteBlock.SetActivatedSilently],
    snippet: raw(block: true, lang: "cs", "Session.Level = next.Name;\nLoadLevel(Player.IntroTypes.Transition);\n...\nentity.SetActivatedSilently(entity.Index == currentIndex);\n...\npublic void WillToggle() { ShiftSize(Collidable ? 1 : -1); }"),
    note: [TransitionRoutine 在过场开头载入新房；Global manager 保留旧 beat，再由 OnLevelStart 只静默初始化新房 block。若这与 WillToggle/SetActiveIndex 同帧，旧房和新房会处在不同的 1px 阶段。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [step; advance_cassette_manager; initialize_cassette_blocks], note: [transition early-return 前仍推进 block 与 global manager；缺口是 runtime Map 不会像 Level.LoadLevel 那样同帧加入下一房 CassetteBlock，故不伪造跨房偏移闭环。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [cassette_manager_keeps_advancing_during_room_transition], note: [回归证明 transition 帧仍从 beat 6 进入 beat 7 并对两颜色执行相反 ShiftSize；它刻意不冒充下一房加载证据。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/other-5.9-screen-transition-cassette-offset.ts; scripts/e2e-real/scenarios/cassette-spinner-parts.ts], symbol: [other-5.9-screen-transition-cassette-offset; TECH_OTHER_5_9_TRANSITION_CASSETTE_OFFSET], note: [独立双房/双颜色 MapPart 已通过 BinaryPacker 编译；必须由真实 Everest 同帧 trace 证明新旧房 block offset 后才能晋级。]),
)
