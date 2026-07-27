#import "../../template.typ": tech, evidence

#tech(
  id: "1.2",
  title-zh: "攀顶小跳",
  title-en: "Climbhop",
  status: "implemented",
  description-zh: [从攀爬状态翻上墙顶时，游戏会自动执行一小段上台动作，并在动作期间锁住水平输入；危险墙顶通常会阻止它。],
  description-en: [Climbing over a ledge triggers a short automatic hop with horizontal control locked, unless hazards on the top make the move unsafe.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.ClimbUpdate; Player.ClimbHop; Player.SlipCheck],
    snippet: raw(block: true, lang: "cs", "else if (SlipCheck()) {\n    ClimbHop();\n    return StNormal;\n}\n...\nhopWaitX = (int) Facing;\nhopWaitXSpeed = (int) Facing * ClimbHopX;\nSpeed.Y = Math.Min(Speed.Y, ClimbHopY);\nforceMoveXTimer = ClimbHopForceTime;\nnoWindTimer = ClimbHopNoWindTime;"),
    note: [SlipCheck 确认手部已越过墙顶后切回 Normal。若仍贴着 Solid，ClimbHop 先保存 100 水平速度并等待身体清出墙沿，同时设置 -120 垂直速度、0.2 秒水平锁定与 0.3 秒抗风窗口。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/types.rs; crates/celeste-physics/src/sim.rs], symbol: [PlayerSnapshot.hop_wait_x; slip_check; climb_update; climb_hop; update_climb_hop_wait]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [climbhop_waits_for_the_body_to_clear_the_ledge_before_horizontal_launch]),
  e2e-evidence: evidence(path: [scripts/e2e-real-collector.mjs], symbol: [mechanics-climbhop], note: [真实关键帧 43 从 Climb 转 Normal，speed=0/-120、hopWaitX=1、forceMoveXTimer=0.2、noWindTimer=0.3；第 47 帧清出墙沿后 hopWaitX=0 且水平速度 89.16665。91 个状态帧九类字段逐帧一致，max position error 0，max speed error 0.000031。]),
  candidate-e2e: none,
)
