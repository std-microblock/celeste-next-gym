# 架构与保真状态

## 当前可运行闭环

- `celeste-physics`：纯函数快照模拟、MessagePack、C ABI、Celeste BinaryPacker 读取。
- `celeste-wasm`：供 Web Worker 使用的 MessagePack WASM 桥。
- `web`：全新逐帧训练界面；Rust WASM 在独立 Worker 中运行，不存在 TypeScript 模拟器回退。WASM 直接解码重新生成的 `CelesteGymPlayground/Playground.bin`。时间线由单 Canvas 绘制和命中测试；输入修改后保留修改帧之前的 state 检查点，后续 state 全部失效，并在跳转、播放或录制时从最近有效检查点按需重算。游戏画面使用从 Celeste 1.4.0.0-fna Gameplay atlas 提取的原版动画和贴图。
- `services/collector`：真实游戏采集服务的协议、校验、超时和 Everest TCP 后端。
- `mods/CelesteGymCollector`：主线程场景加载、逐帧输入覆盖与反射快照采集。

## 保真声明

当前版本的保真标签为 `source_informed_subset`。常数取自随附的 Celeste 1.4.0.0 FNA 源码，已实现 Normal、Dash、Climb 的首个确定性子集、像素步进轴向碰撞、JumpThru、DreamBlock 几何，以及 Water/Wind 的占位交互。

它尚未达到 1:1，尤其缺少完整的角落修正、DashSlide、所有特殊状态、实体协程、平台携带、完整输入缓冲语义及与真实游戏逐帧数据的自动回归。未支持的玩家状态会返回明确错误，不会静默降级。

## ABI 修订

规格草案同时把 `inputs_len` 描述为“序列帧数”和序列化缓冲区参数。C ABI 必须知道缓冲区字节长度才能安全读取，因此实现将 `inputs_len` 定义为 MessagePack 缓冲区字节数；要执行的帧数继续由 `frames` 提供。

输出缓冲区由调用方分配，但当前 MessagePack 解码和模拟轨迹在函数内部仍会分配内存。“调用方零分配”已满足，“核心绝对零分配”尚未满足。

## 后续对标门槛

只有在真实 Everest 采集后端接通，并让每项技巧的真实轨迹在位置/速度误差 `<= 0.01` 的回归测试中通过后，才应升级保真声明。

## 已建立的真实门禁

`scripts/e2e-real-collector.mjs` 当前验证原版第一章首房间的三条轨迹：跑动 30 帧、跳跃 45 帧、水平冲刺 12 帧。它通过 HTTP MessagePack → 采集服务 → TCP Mod → Celeste/Everest → 126 字段真实快照 → Rust 差分比较的完整链路执行，三条轨迹均满足误差 `<= 0.01`。

这仍不是全机制证明；墙体技巧、特殊状态、实体交互、死亡/重生和任意地图动态挂载尚需按同一门禁扩展。
