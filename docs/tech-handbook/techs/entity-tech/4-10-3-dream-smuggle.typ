#import "../../template.typ": tech, evidence

#tech(
  id: "4.10.3",
  title-zh: "Dream Smuggle 梦块携物",
  title-en: "Dream Smuggle",
  status: "implemented",
  description-zh: [在梦块入口前冲向投掷物并抓取，可把 Theo 或水母带进通常不能携物进入的梦块，并在出口继续操作。],
  description-en: [Dashing into and grabbing a throwable immediately before a dream block carries it through a state that normally forbids held items.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs / Source/TheoCrystal.cs],
    symbol: [Player.PickupCoroutine / Player.DreamDashBegin / TheoCrystal.Hold.Pickup],
    snippet: raw(block: true, lang: "cs", "yield return 0.16f;\nSpeed = oldSpeed;\nSpeed.Y = Math.Min(Speed.Y, 0f);\nvarJumpTimer = oldVarJumpTimer;\nStateMachine.State = 0;"),
    note: [Pickup 暂停玩家 0.16 秒后恢复进入抓取前的速度与可变跳计时，却不重置仍在流逝的 dashAttackTimer；回到 Normal 后，残留 DashAttack 与入口 DreamBlock 相交即可进入 DreamDash，而 Theo 的 Hold 仍归玩家。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [try_pickup_theo / pickup_update / interact], note: [实现保存 oldSpeed/varJumpTimer、源匹配的 0.16 秒 Tween 边界与独立 Theo 持有状态，并允许 lingering dash attack 进入 DreamBlock。]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [dream_smuggle_keeps_theo_through_pickup_and_lingering_attack_entry / dash_pickup_cancels_into_source_pickup_tween_and_restores_speed]),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [entity-4.10.3-dream-smuggle],
    note: [真实独立 MapPart 共 81 个状态，依次观测 Pickup、持 Theo 的 DreamDash 内部状态与持 Theo 的出口状态；position/speed 最大误差均为 0，其余七类字段也逐帧一致，`holding_theo` 从 Collector Mod 到服务与 E2E 比较全链路贯通。],
  ),
  candidate-e2e: none,
)
