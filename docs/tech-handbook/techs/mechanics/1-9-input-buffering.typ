#import "../../template.typ": tech, evidence

#tech(
  id: "1.9",
  title-zh: "输入缓冲",
  title-en: "Input Buffering",
  status: "implemented",
  description-zh: [暂时无法执行的动作通常会保留 5 帧；若窗口内条件变为合法且按钮仍满足要求，动作会在首个可执行帧触发。],
  description-en: [Most actions can be buffered for five frames and fire on the first legal frame if their input remains valid.],
  source-evidence: evidence(path: [Source/Player/Player.cs], symbol: [Player.NormalUpdate; Input.Jump.Pressed], note: [NormalUpdate 在接地判定后消费仍处于 VirtualButton 缓冲期的跳跃输入。]),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [step; tick_timers; normal_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [buffered_jump_fires_on_the_first_grounded_update]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [buffered-jump], note: [落地前按跳，首次接地更新后起跳；九类字段最大误差 0。]),
  candidate-e2e: none,
)
