#let status-style(status) = if status == "implemented" {
  (label: "已实现", color: rgb("#166534"), background: rgb("#dcfce7"))
} else {
  (label: "未实现 / Not implemented", color: rgb("#991b1b"), background: rgb("#fee2e2"))
}

#let missing = [#text(fill: rgb("#64748b"))[未提供 / Not provided]]

#let coverage-summary(expected: 120) = context {
  let entries = query(<tech-entry>)
  let implemented = entries.filter(entry => entry.value.status == "implemented").len()
  let total = entries.len()

  assert(
    total == expected,
    message: "expected " + str(expected) + " techniques, found " + str(total),
  )

  [#implemented / #total]
}

#let evidence(path: none, symbol: none, snippet: none, note: none) = block(
  width: 100%,
  inset: 8pt,
  radius: 4pt,
  fill: rgb("#f8fafc"),
  stroke: 0.5pt + rgb("#cbd5e1"),
  [
    #if path != none [*文件:* #path #linebreak()]
    #if symbol != none [*符号:* #symbol #linebreak()]
    #if note != none [*说明:* #note]
    #if snippet != none [#v(5pt)#snippet]
  ],
)

#let evidence-cell(title, value) = block(
  width: 100%,
  inset: 9pt,
  radius: 5pt,
  stroke: 0.6pt + rgb("#cbd5e1"),
  [
    *#title*
    #v(5pt)
    #if value == none { missing } else { value }
  ],
)

#let tech(
  id: none,
  title-zh: none,
  title-en: none,
  status: none,
  description-zh: none,
  description-en: none,
  source-evidence: none,
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
) = {
  let badge = status-style(status)
  [
    #metadata((id: id, status: status)) <tech-entry>

    == #id　#title-zh
    #text(size: 12pt, fill: rgb("#475569"), style: "italic")[#title-en]

    #v(5pt)
    #box(
      inset: (x: 9pt, y: 5pt),
      radius: 999pt,
      fill: badge.background,
      stroke: 0.7pt + badge.color,
      text(fill: badge.color, weight: "bold", badge.label),
    )

    #if candidate-e2e != none [
      #h(7pt)
      #box(
        inset: (x: 9pt, y: 5pt),
        radius: 999pt,
        fill: rgb("#fef3c7"),
        stroke: 0.7pt + rgb("#a16207"),
        text(fill: rgb("#854d0e"), [候选 E2E：#candidate-e2e（尚不能证明实现）]),
      )
    ]

    === 中文说明
    #description-zh

    === English description
    #description-en

    === 来源
    #grid(
      columns: (1fr),
      gutter: 8pt,
      evidence-cell([源码], source-evidence),
      evidence-cell([Rust 实现], rust-evidence),
      evidence-cell([回归测试], test-evidence),
      evidence-cell([真实 E2E], e2e-evidence),
    )

    #if status != "implemented" [
      #v(6pt)
      #text(size: 8.5pt, fill: rgb("#64748b"))[
        完成条件：上游源码审计、Rust 真实机制、回归测试、真实 Everest E2E 四类证据必须全部存在；E2E 需比较 position、speed、state、facing、dashes、stamina、grounded、ducking、death，数值容差不超过 0.01。
      ]
    ]
  ]
}
