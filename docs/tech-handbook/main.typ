#import "template.typ": evidence

#set document(
  title: "Celeste Next Gym 技巧手册 / Tech Handbook",
  author: "Celeste Next Gym contributors",
)
#set page(
  paper: "a4",
  margin: (x: 22mm, y: 20mm),
  numbering: "1 / 1",
  number-align: center + bottom,
)
#set text(
  font: ("Noto Sans CJK SC", "Microsoft YaHei", "Arial"),
  lang: "zh",
  size: 10pt,
)
#set par(justify: true, leading: 0.72em)
#set heading(numbering: none)
#show heading.where(level: 1): it => pagebreak(weak: true) + block(above: 0pt, below: 16pt, it)
#show heading.where(level: 2): it => block(above: 18pt, below: 8pt, it)
#show raw: set text(font: "Consolas", size: 8pt)

#align(center)[
  #v(28mm)
  #text(size: 25pt, weight: "bold")[Celeste Next Gym 技巧手册]
  #v(4pt)
  #text(size: 17pt, fill: rgb("#475569"))[Tech Handbook]
  #v(16pt)
  #text(size: 11pt)[固定基线：Celeste Wiki Tech 页面 2026-05-28 文本快照]
  #v(3pt)
  #text(size: 10pt, fill: rgb("#64748b"))[Pinned baseline: Celeste Wiki Tech snapshot revised 2026-05-28]
  #v(20pt)
  #box(
    width: 72%,
    inset: 14pt,
    radius: 7pt,
    fill: rgb("#f8fafc"),
    stroke: 0.8pt + rgb("#94a3b8"),
    [
      *当前覆盖 / Current coverage*

      #text(size: 19pt, weight: "bold", fill: rgb("#991b1b"))[0 / 120]

      所有技巧在四类证据齐全前均明确标记为“未实现”。

      Every technique remains “Not implemented” until all four evidence classes are complete.
    ],
  )
]

#pagebreak()
= 阅读与维护说明 / Reading and maintenance

本手册既是面向玩家的双语技巧说明，也是实现覆盖审计。每个技巧独立存放在一个 Typst 文件中，并由本入口统一包含。来源描述经过摘要和双语改写；实现结论只以代码与真实游戏证据为准。

This handbook is both a bilingual player-facing guide and an implementation audit. Every technique lives in its own Typst file and is included by this entry point. Descriptions are summarized and rewritten bilingually; implementation verdicts depend only on code and real-game evidence.

只有以下证据全部存在时，技巧才能标记为“已实现”：

+ 对照上游 `Player.cs` 及相关实体源码，并记录入口和关键代码片段；
+ Rust 中实现真实机制，而不是隐藏状态或测试捷径；
+ 有覆盖关键帧、速度与状态不变量的回归测试；
+ 有标准实体和正常输入触发的真实 Celeste/Everest E2E，完整比较九类字段且误差不超过 0.01。

候选场景只是一条待核证线索，不计入完成数。`FinalBoss` 与玩家 `Attract` 状态是明确产品排除项，不进入 120 项分母。

#outline(title: [目录 / Contents], depth: 2, indent: auto)

#pagebreak()
#include "techs.typ"
