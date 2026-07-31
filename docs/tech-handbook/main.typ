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
]

#pagebreak()

#outline(title: [目录 #h(5pt) #badge([CONTENTS])], depth: 2, indent: auto)

#pagebreak()
#include "techs.typ"
