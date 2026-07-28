#import "../../template.typ": tech, evidence

#tech(
  id: "4.16",
  title-zh: "岩浆 Neutral",
  title-en: "Lava Neutrals",
  status: "unimplemented",
  description-zh: [岩浆受伤区前有 1 像素实体边缘，可在该像素上缓冲 Neutral、攀跳、墙跳、墙反等墙面动作。],
  description-en: [RisingLava and SandwichLava are camera/core-driven PlayerCollider hazards whose 8×11 player body and 8×9 hurtbox leave a one-pixel wall-action lip before death.],
  source-evidence: evidence(
    path: [Celeste/RisingLava.cs; Celeste/SandwichLava.cs],
    symbol: [RisingLava.Added / RisingLava.Update / SandwichLava.Update / Player.orig_Update],
    snippet: raw(block: true, lang: "cs", "Collider = new Hitbox(340f, 120f);\nY = level.Bounds.Bottom + 16;\nX = level.Camera.X;\nY += -30f * multiplier * Engine.DeltaTime;\n// player: 8x11 body, 8x9 hurtbox at (-4,-11)\nCollider was = Collider;\nCollider = hurtbox;\nforeach (PlayerCollider pc in Scene.Tracker.GetComponents<PlayerCollider>()) pc.Check(this);"),
    note: [RisingLava 从房间底部 +16 出生、每帧锁定 camera X，并以 -30×自适应倍率上升；SandwichLava 使用相隔 280px 的两个 340×120 collider，hot/cold 分别 -20/+20 px/s，另有 Waiting、persistent reuse 与 leaving。玩家移动/墙面动作使用 8×11 body，而 PlayerCollider 致死阶段临时换成同顶部、少 2px 高的 8×9 hurtbox，因此底部恰有 1px 可动作安全唇。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/map.rs / crates/celeste-physics/src/types.rs / crates/celeste-physics/src/sim.rs],
    symbol: [EntityKind.RisingLava / EntityKind.SandwichLava / RisingLavaSnapshot / SandwichLavaSnapshot / advance_rising_lavas / advance_sandwich_lavas / current_player_hurt_rect],
    note: [BinaryPacker、fixture 与 runtime 均有独立 Rising/Sandwich 实体；snapshot 保存 camera、core mode、JustRespawned、Waiting、persistent reuse、leaving 和双 collider 状态。PlayerCollider 只在交互阶段采用 hurtbox，墙面动作仍用 body，保留真实 1px 边缘。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/map.rs / crates/celeste-physics/src/sim.rs],
    symbol: [vanilla_core_lavas_round_trip_with_source_colliders / rising_lava_uses_camera_x_and_source_adaptive_rise_speed / sandwich_lava_waiting_core_mode_and_transition_lifecycle_match_source / lava_player_collider_preserves_the_one_pixel_safe_lip / rising_lava_safe_lip_accepts_a_buffered_neutral_climb_jump],
    note: [回归覆盖地图 roundtrip、RisingLava camera/自适应速度、SandwichLava hot/cold 与 transition 生命周期、body/hurtbox 边界，以及第 169 帧安全唇 Neutral 的 -105 Y 速度和 wallboost 窗口。],
  ),
  e2e-evidence: none,
  candidate-e2e: evidence(
    path: [scripts/e2e-real/scenarios/core-heart-squish-parts.ts / scripts/e2e-real/scenarios/playground/entity-4.16-lava-neutral.ts],
    symbol: [tech.entity-4.16-lava-neutral / entity-4.16-lava-neutral],
    note: [独立 MapPart 在训练场右墙加入 vanilla RisingLava；候选在第 169 帧安全唇缓冲 Neutral，真实 Everest 与 Rust 均得到 -105 垂直速度、wallBoostTimer 且未死亡，221 个状态的 position/speed 最大误差为 0。一次 Player.cs 驱动的初始化修正把 Facing 放到 Climb 状态入口之前；唯一复跑仍在 frame 1 出现 stamina 首差（Rust 109.833、Everest 110），超过 0.01，故保持 unimplemented。],
  ),
)
