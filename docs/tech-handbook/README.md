# Celeste Next Gym 技巧手册

`main.typ` 是双语技巧手册入口，`techs/` 下每个技巧一个 Typst 文件，`techs.typ` 负责按固定的 121 项清单统一 `include`。`techs.typ` 的顺序和这 121 个条目文件是唯一权威覆盖清单与证据载体。`template.typ` 统一控制条目版式、证据卡片与媒体展示，不要在单个条目里复制样式。

编译：

```text
typst compile docs/tech-handbook/main.typ .tmp/tech-handbook.pdf
```

前 120 项基线来自 Celeste Wiki Tech 页面最后修订于 2026-05-28 的本地快照；第 121 项是用户明确要求加入的冲刺方向延迟采样机制。不要自行刷新在线 Wiki；只有用户明确要求时才能更新清单。`FinalBoss` 与玩家 `Attract` 状态是产品排除项，不进入 121 项分母。

## 标记完成的规则

默认状态是 `unimplemented`。只有上游源码、Rust 实现、回归测试、真实 Everest E2E 四类证据齐全后，才可以把对应文件改为 `implemented`。证据应使用 `template.typ` 中的 `evidence` 结构记录文件、符号、说明和关键代码片段。

每项 `source-evidence` 必须放入能决定该技巧行为的短代码片段，并用几句话简要解释状态回调顺序、常量、计时或碰撞行为。不要写成长篇源码审计。Rust、单测和 E2E 证据保持具体到符号或场景名；E2E 必须是正常输入和标准实体触发，不能以候选场景代替。

示例：

```typst
#import "../../template.typ": tech, evidence

#tech(
  id: "2.2",
  title-zh: "Super 冲刺跳",
  title-en: "Superdash (Super)",
  status: "implemented",
  description-zh: [中文摘要。],
  description-en: [English summary.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.Jump],
    note: [说明对应状态、常量和调用顺序。],
    snippet: raw(block: true, lang: "cs", "关键上游代码片段"),
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [Simulator::jump],
    snippet: raw(block: true, lang: "rust", "关键 Rust 代码片段"),
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [superdash_matches_source_timing],
  ),
  e2e-evidence: evidence(
    path: [scripts/e2e-real-collector.mjs],
    symbol: [super],
    note: [九类字段全部比较，最大数值误差 <= 0.01。],
  ),
)
```

真实 E2E 必须比较 position、speed、state、facing、dashes、stamina、grounded、ducking、death，数值误差不得超过 `0.01`。

每次修改条目状态时，首页会直接查询各条目的 `status` 并自动统计覆盖数；编译时也会断言 `techs.typ` 仍恰好 include 121 个条目，不再手工维护计数。

## 图片、GIF 与视频

媒体文件统一放在 `docs/tech-handbook/media/`。PDF 不能播放动画或视频，因此 GIF 使用静态预览帧，视频使用封面图；如提供 `url`，整张卡片和“打开媒体”提示都可点击。一个条目可以传入一张或多张媒体卡：

```typst
#import "../../template.typ": tech, evidence, media-gallery, media-item

#tech(
  // 其余字段省略
  media: media-gallery((
    media-item(
      preview: "media/2-2-super.webp",
      caption: [Super 的输入与位移演示],
      url: "https://example.com/super.mp4",
      kind: "video",
      alt: "Superdash demonstration",
    ),
    media-item(
      preview: "media/2-2-super-frames.webp",
      caption: [关键帧拆解],
      kind: "gif",
    ),
  )),
)
```

`kind` 可选 `"image"`、`"gif"` 或 `"video"`。为保证打印和离线阅读质量，预览图建议使用 16:9 的 WebP/PNG，宽度至少 1280 px；即使原始素材是 GIF，也建议单独导出一张代表性静态帧。
