# Celeste Next Gym 技巧手册

`main.typ` 是双语技巧手册入口，`techs/` 下每个技巧一个 Typst 文件，`techs.typ` 负责按固定的 120 项基线统一 `include`。

编译：

```text
typst compile docs/tech-handbook/main.typ .tmp/tech-handbook.pdf
```

`docs/tech-coverage.md` 继续固定 120 项编号基线；每个技巧的双语摘要、状态和证据直接维护在对应的 Typst 文件中。

## 标记完成的规则

默认状态是 `unimplemented`。只有上游源码、Rust 实现、回归测试、真实 Everest E2E 四类证据齐全后，才可以把对应文件改为 `implemented`。证据应使用 `template.typ` 中的 `evidence` 结构记录文件、符号、说明和关键代码片段。

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
