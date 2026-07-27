#import "../../template.typ": tech, evidence

#tech(
  id: "5.7",
  title-zh: "暂停缓冲",
  title-en: "Pause Buffering",
  status: "unimplemented",
  description-zh: [暂停菜单属于游戏外时间控制：暂停时 Level/Player 更新停止，但 Monocle 输入节点继续在引擎层采样；在解除暂停前约六帧按下动作，可让 VirtualButton 的缓冲在恢复后的第一帧被 Player 消费。按当前产品决定只记录机制，不在 Rust 模拟器内实现。],
  description-en: [Pausing is out-of-game time control: Level and Player updates stop while Monocle input continues sampling at the engine layer. Presses made roughly six frames before unpausing can remain in a VirtualButton buffer for the first resumed Player frame. This is documented but intentionally not implemented in the Rust simulator.],
  source-evidence: evidence(
    path: [Monocle/Engine.cs; Monocle/VirtualButton.cs; Celeste/Level.cs; Source/Player/Player.cs],
    symbol: [Engine.Update; VirtualButton.Update; Level.Pause; Level.Update; Player.Update],
    note: [关键边界不在 Player 状态机，而在引擎输入节点与暂停 Scene 更新的分层：暂停阻止 Level/Player 帧推进，输入仍可进入 VirtualButton 缓冲；解除暂停后的首个 Player.Update 看到尚未过期的 Pressed。具体可用窗口受 VirtualButton.BufferTime 与原始时间递减顺序控制。],
  ),
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
