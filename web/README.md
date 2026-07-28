# Celeste Next Gym Web

全新逐帧训练界面。浏览器端物理只通过 Rust WASM Worker 运行，没有 TypeScript 模拟器或回退实现。

- state 按帧缓存；跳转只从最近有效检查点计算到目标帧。
- 修改输入帧 `N` 后，仅令 `N + 1` 及之后的 state 失效。
- 播放和录制期间按需增量计算，不预跑整条时间线；地图只在切换时传入 Worker 并在 WASM 内缓存解码结果。
- 时间线直接编辑上、下、左、右、跳跃、冲刺、抓取，支持操作块选中、复制、粘贴、删除和跨轨拖动。
- 时间线获得焦点后，左右方向键移动当前帧；选中操作块后，左右方向键移动该操作。Ctrl + 滚轮缩放，普通滚轮仅滚动时间线。
- 时间线由单个 Canvas 绘制和命中测试，不为每一帧创建 DOM 节点。
- 默认键位：W / S / A / D / L / ; / '，可在界面中重新绑定。
- 支持浏览器标准映射手柄：A / × 跳跃、X / □ 冲刺、肩键或扳机抓取；方向输入可在“控制”中选择左摇杆或十字键。
- 地图由 `celeste-wasm 0.2.0` 在 Worker 内直接解码重新生成的 `CelesteGymPlayground/Playground.bin`。
- 场景使用从 Celeste 1.4.0.0-fna Gameplay atlas 提取的原版角色动画和实体/地形贴图。

```bash
npm install
npm test
npm run build
```

重新生成 WASM 浏览器包：

```bash
node ../scripts/build-wasm.mjs
```

原版视觉素材的本地来源记录见 `public/assets/original/README.md`。这些文件用于首版视觉占位，后续可原路径替换为重绘资源。WASM 绑定生成在 `src/wasm`，由 Vite 和 Worker 一起打包。
