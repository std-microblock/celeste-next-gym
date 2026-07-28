#let badge(
  body,
  fill: rgb("#edf2f7"),
  color: rgb("#475569"),
  stroke: none,
  size: 7.5pt,
) = box(
  inset: (x: 6pt, y: 3pt),
  radius: 999pt,
  fill: fill,
  stroke: stroke,
  text(size: size, weight: "bold", fill: color, body),
)

#let status-style(status) = if status == "implemented" {
  (zh: "已实现", en: "IMPLEMENTED", color: rgb("#166534"), background: rgb("#dcfce7"))
} else if status == "product-excluded" {
  (zh: "产品排除", en: "PRODUCT EXCLUDED", color: rgb("#7c2d12"), background: rgb("#ffedd5"))
} else {
  (zh: "未实现", en: "NOT IMPLEMENTED", color: rgb("#991b1b"), background: rgb("#fee2e2"))
}

#let missing = [
  #badge([未提供], fill: rgb("#f1f5f9"), color: rgb("#64748b"))
  #h(4pt)
  #badge([NOT PROVIDED], fill: rgb("#f1f5f9"), color: rgb("#64748b"))
]

#let coverage-summary(expected: 120, expected_excluded: 5) = context {
  let entries = query(<tech-entry>)
  let implemented = entries.filter(entry => entry.value.status == "implemented").len()
  let excluded = entries.filter(entry => entry.value.status == "product-excluded").len()
  let total = entries.len()
  let product-total = total - excluded

  assert(
    total == expected,
    message: "expected " + str(expected) + " techniques, found " + str(total),
  )
  assert(
    excluded == expected_excluded,
    message: "expected " + str(expected_excluded) + " product exclusions, found " + str(excluded),
  )

  [
    #implemented / #product-total #h(4pt) #badge([产品覆盖], fill: rgb("#e2e8f0"), color: rgb("#475569"), size: 7pt)
    #h(9pt)
    #total - #excluded = #product-total #h(4pt) #badge([固定基线／排除／分母], fill: rgb("#e2e8f0"), color: rgb("#475569"), size: 7pt)
  ]
}

#let identifier(value) = {
  show regex("[_;/]"): it => [#it#text("\u{200b}")]
  value
}

#let evidence-row(zh, en, value, technical: false) = grid(
  columns: (auto, auto, 1fr),
  gutter: 5pt,
  align: top,
  badge(zh, fill: rgb("#e2e8f0"), color: rgb("#334155")),
  badge(en, fill: rgb("#f1f5f9"), color: rgb("#64748b")),
  block(width: 100%)[
    #set par(justify: false)
    #if technical { identifier(value) } else { value }
  ],
)

#let evidence(path: none, symbol: none, snippet: none, note: none) = block(
  width: 100%,
  breakable: false,
  inset: 8pt,
  radius: 4pt,
  fill: rgb("#f8fafc"),
  stroke: 0.5pt + rgb("#cbd5e1"),
  [
    #if path != none [
      #evidence-row([文件], [FILE], path, technical: true)
      #v(5pt)
    ]
    #if symbol != none [
      #evidence-row([符号], [SYMBOL], symbol, technical: true)
      #v(5pt)
    ]
    #if note != none [
      #evidence-row([说明], [NOTE], note)
    ]
    #if snippet != none [#v(5pt)#snippet]
  ],
)

#let evidence-cell(title-zh, title-en, value) = block(
  width: 100%,
  breakable: false,
  inset: 9pt,
  radius: 5pt,
  stroke: 0.6pt + rgb("#cbd5e1"),
  [
    #badge(title-zh, fill: rgb("#dbeafe"), color: rgb("#1e40af"), size: 8pt)
    #h(5pt)
    #badge(title-en, fill: rgb("#eff6ff"), color: rgb("#475569"))
    #v(6pt)
    #if value == none { missing } else { value }
  ],
)

#let media-item(
  preview: none,
  caption: none,
  url: none,
  kind: "image",
  alt: "Technique media preview",
) = (
  preview: preview,
  caption: caption,
  url: url,
  kind: kind,
  alt: alt,
)

#let media-card(item) = {
  let kind-label = if item.kind == "video" { "VIDEO" } else if item.kind == "gif" { "GIF" } else { "IMAGE" }
  let preview = if item.preview == none {
    align(center + horizon, text(size: 9pt, fill: rgb("#94a3b8"))[媒体预览])
  } else {
    image(item.preview, width: 100%, height: 100%, fit: "cover", alt: item.alt)
  }
  let body = block(
    width: 100%,
    breakable: false,
    inset: 8pt,
    radius: 5pt,
    fill: rgb("#f8fafc"),
    stroke: 0.6pt + rgb("#cbd5e1"),
    [
      #block(width: 100%, height: 85pt, clip: true, fill: rgb("#e2e8f0"), preview)
      #v(6pt)
      #badge(kind-label, fill: rgb("#e2e8f0"), color: rgb("#334155"))
      #if item.caption != none [#h(5pt)#item.caption]
      #if item.url != none [#h(5pt)#badge([打开媒体 ↗], fill: rgb("#dbeafe"), color: rgb("#1d4ed8"))]
    ],
  )
  if item.url == none { body } else { link(item.url, body) }
}

#let media-gallery(items) = grid(
  columns: if items.len() == 1 { (1fr,) } else { (1fr, 1fr) },
  gutter: 8pt,
  ..items.map(media-card),
)

#let tech(
  id: none,
  title-zh: none,
  title-en: none,
  status: none,
  description-zh: none,
  description-en: none,
  media: none,
  source-evidence: none,
  rust-evidence: none,
  test-evidence: none,
  e2e-evidence: none,
  candidate-e2e: none,
) = {
  let status-badge = status-style(status)
  [
    #metadata((id: id, status: status)) <tech-entry>

    == #id　#title-zh
    #text(size: 12pt, fill: rgb("#475569"), style: "italic")[#title-en]

    #v(5pt)
    #badge(status-badge.zh, fill: status-badge.background, color: status-badge.color, stroke: 0.7pt + status-badge.color, size: 8pt)
    #h(5pt)
    #badge(status-badge.en, fill: status-badge.background, color: status-badge.color, stroke: 0.7pt + status-badge.color)

    #if candidate-e2e != none [
      #h(7pt)
      #badge([候选 E2E], fill: rgb("#fef3c7"), color: rgb("#854d0e"), stroke: 0.7pt + rgb("#a16207"), size: 8pt)
      #h(5pt)
      #badge([CANDIDATE], fill: rgb("#fef3c7"), color: rgb("#854d0e"), stroke: 0.7pt + rgb("#a16207"))
      #h(5pt)
      #candidate-e2e
      #h(3pt)
      #text(size: 8pt, fill: rgb("#854d0e"))[(尚不能证明实现)]
    ]

    === 中文说明 #h(5pt) #badge([DESCRIPTION])
    #description-zh

    === English description #h(5pt) #badge([ENGLISH])
    #description-en

    #if media != none [
      === 演示媒体 #h(5pt) #badge([MEDIA])
      #media
    ]

    === 实现 #h(5pt) #badge([EVIDENCE])
    #grid(
      columns: (1fr),
      gutter: 8pt,
      evidence-cell([源码], [SOURCE], source-evidence),
      evidence-cell([Rust 实现], [RUST], rust-evidence),
      evidence-cell([回归测试], [REGRESSION TEST], test-evidence),
      evidence-cell([真实 E2E], [EVEREST E2E], e2e-evidence),
    )

    #if status == "unimplemented" [
      #v(6pt)
      #text(size: 8.5pt, fill: rgb("#64748b"))[
        完成条件：上游源码审计、Rust 真实机制、回归测试、真实 Everest E2E 四类证据必须全部存在；E2E 需比较 position、speed、state、facing、dashes、stamina、grounded、ducking、death，数值容差不超过 0.01。
      ]
    ]
  ]
}
