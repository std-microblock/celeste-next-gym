#import "../../template.typ": tech, evidence

#tech(
  id: "4.29",
  title-zh: "Springboost Cancel",
  title-en: "Springboost cancel",
  status: "unimplemented",
  description-zh: [携物触发弹簧后立刻丢出并重抓，可取消部分水平弹簧动量，换取更偏纵向的运动。],
  description-en: [Throwing and quickly regrabbing a holdable after a spring can cancel horizontal spring momentum while retaining more vertical motion.],
  source-evidence: evidence(
    path: [Source/Spring.cs / Source/TheoCrystal.cs / Source/Glider.cs / Source/Player/Player.cs],
    symbol: [Spring.OnHoldable / TheoCrystal.HitSpring / Glider.HitSpring / Player.PickupCoroutine],
    snippet: raw(block: true, lang: "cs", "if (h.HitSpring(this)) BounceAnimate();\n...\nSpeed.X *= 0.5f;\nSpeed.Y = -160f;\nnoGravityTimer = 0.15f;"),
    note: [Spring 直接把碰撞分派给 Holdable.HitSpring。未被持有的 Glider 在地板弹簧上把 X 减半、Y 设为 -160，并设置 0.15 秒无重力；Rust 按实体后更新顺序执行 Glider 移动再检查 Spring。后续丢出／重抓取消链仍待真实采集。],
  ),
  rust-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [advance_gliders / hit_glider_spring / release_glider / pickup_update]),
  test-evidence: evidence(path: [crates/celeste-physics/src/sim.rs], symbol: [floor_spring_launches_unheld_glider_after_actor_movement / glider_spring_no_gravity_keeps_its_final_source_frame / springboost_cancel_reverses_into_the_rising_glider_for_regrab], note: [回归锁定 Spring 的 -160 Y 速度和 0.15 秒无重力计时；最后一个正计时帧仍不施加重力，之后才可进入取消重抓链。]),
  e2e-evidence: none,
  candidate-e2e: evidence(path: [scripts/e2e-real/scenarios/playground/entity-4.29-springboost-cancel.ts], symbol: [entity-4.29-springboost-cancel], note: [2026-07-28 真实 Everest 重跑确认 release→Spring→regrab 语义，但第 103 帧 Rust 已在 (126,460) 进入 Pickup，Everest 为 (126,462)、速度 (0,160)、Normal；首差为位置 2／速度 160，最大位置／速度误差 8／160，保持未实现。]),
)
