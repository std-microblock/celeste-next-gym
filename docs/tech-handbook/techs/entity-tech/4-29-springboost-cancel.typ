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
    note: [Spring 直接把碰撞分派给 Holdable.HitSpring。未被持有的 Theo 与 Glider 在地板弹簧上都把 X 减半、Y 设为 -160，并设置 0.15 秒无重力；之后投掷与 Pickup 的缓存／恢复顺序形成取消窗口。当前 Rust 只实现 Spring→Player 回调，没有 Spring→Theo/Glider 的实体碰撞和 HitSpring runtime，故不能建立回归或真实 E2E。],
  ),
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
)
