#import "template.typ": badge, coverage-summary

#set document(
  title: "Celeste Next Gym 技巧手册 / Tech Handbook",
  author: "Celeste Next Gym contributors",
)
#set page(
  paper: "a4",
  margin: (x: 22mm, y: 20mm),
  numbering: "1 · 1",
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
  #badge([固定基线], fill: rgb("#e2e8f0"), color: rgb("#334155"))
  #h(5pt)
  #badge([PINNED BASELINE], fill: rgb("#e2e8f0"), color: rgb("#334155"))
  #v(7pt)
  #text(size: 10pt)[Celeste Wiki Tech · 2026-05-28]
  #v(20pt)
  #box(
    width: 72%,
    inset: 14pt,
    radius: 7pt,
    fill: rgb("#f8fafc"),
    stroke: 0.8pt + rgb("#94a3b8"),
    [
      #badge([当前覆盖], fill: rgb("#e2e8f0"), color: rgb("#334155"))
      #h(5pt)
      #badge([CURRENT COVERAGE], fill: rgb("#e2e8f0"), color: rgb("#334155"))

      #v(9pt)
      #text(size: 19pt, weight: "bold", fill: rgb("#991b1b"))[#coverage-summary()]

      产品范围内的技巧在四类证据齐全前均明确标记为“未实现”；5 个条目明确标记为“产品排除”。

      Every in-scope technique remains “Not implemented” until all four evidence classes are complete; five entries are explicitly product-excluded.
    ],
  )
]

#pagebreak()
= 阅读与维护说明 #h(5pt) #badge([READING AND MAINTENANCE])

本手册既是面向玩家的双语技巧说明，也是实现覆盖审计。每个技巧独立存放在一个 Typst 文件中，并由本入口统一包含。来源描述经过摘要和双语改写；实现结论只以代码与真实游戏证据为准。

This handbook is both a bilingual player-facing guide and an implementation audit. Every technique lives in its own Typst file and is included by this entry point. Descriptions are summarized and rewritten bilingually; implementation verdicts depend only on code and real-game evidence.

只有以下证据全部存在时，技巧才能标记为“已实现”：

+ 对照上游 `Player.cs` 及相关实体源码，并记录入口和关键代码片段；
+ Rust 中实现真实机制，而不是隐藏状态或测试捷径；
+ 有覆盖关键帧、速度与状态不变量的回归测试；
+ 有标准实体和正常输入触发的真实 Celeste/Everest E2E，完整比较九类字段且误差不超过 0.01。

候选场景只是一条待核证线索，不计入完成数。`5.4`、`5.7`、`5.10`、`5.11` 与 `4.19` 是产品排除项，固定基线 120 项减去这 5 项后，产品覆盖分母为 115；`FinalBoss` 与玩家 `Attract` 状态不在这 120 项中。保存不是权威 120 项清单中的技巧，因而不影响该分母。

#outline(title: [目录 #h(5pt) #badge([CONTENTS])], depth: 2, indent: auto)

#pagebreak()
#include "techs.typ"
