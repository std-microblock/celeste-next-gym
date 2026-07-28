# 写一个训练技巧

训练定义全部是浏览器直接导入的 TypeScript；不需要先写 JSON，也没有运行时 `fetch` / `readFile` / `writeFile`。

## 文件位置

- 每个技巧一个文件：`web/src/training/techniques/<技巧>.ts`
- 总目录：`web/src/training/catalog.ts`
- 可复用工具：`web/src/training/helpers.ts`

新增技巧时，在独立文件中定义一个 `TrainingTechnique`，然后在 `catalog.ts` 里静态 `import` 并放进 `trainingCatalog`。

## 一个技巧与 Variant

一个技巧应包含 2–3 个实战 Variant。Variant 是独立的训练对象，拥有自己的：

- 地图和实体；
- 初始玩家状态；
- Fuzz 输入空间与成功条件；
- 教学步骤和提示；
- 自动慢放配置。

例如 Hyper 当前包含：

1. 地面中间有断层，要求跨到另一侧；
2. 越过一片尖刺；
3. 从泡泡状态开始接 Hyper。

可以复用 `room()`、`snapshot()`、`hold()`、`press()`，也可以为同一类技巧编写更专门的 helper，例如 `hyperVariant()`。

## 必须通过 simulator

训练不能只靠肉眼设计。每个 Variant 都必须把 `initial`、`map` 和 `document.fuzz` 交给 `WasmClient.fuzzSearch()`，并确认至少存在一个成功候选。

运行：

```bash
cd web
npm test
```

`web/scripts/validate-training.mts` 会直接导入 `trainingCatalog`，用真实 `celeste-wasm` 遍历全部 Variant 的 `validationFuzz`，确认：

- Fuzz 返回成功候选；
- 最佳候选不死亡；
- 成功条件确实覆盖场景目的，例如跨过断层、越过尖刺或从实体状态退出；
- 初始位置与碰撞地形对齐。

## 关联技巧

`related` 填技巧 ID，例如：

```ts
related: ['wavedash', 'super']
```

训练选择器会显示“你还可以看看”。只有已经加入 `trainingCatalog` 的技巧才可切换；尚未实现的关联项仍可作为后续内容提示。

## 完成流程

1. 在独立技巧文件内添加 2–3 个 Variant。
2. 将技巧静态导入 `trainingCatalog`。
3. 为场景写明确的 Fuzz `success`，不要只检查速度。
4. 用 simulator 验证所有 Variant 可完成。
5. 运行 `npm test` 和 `npm run build`。
