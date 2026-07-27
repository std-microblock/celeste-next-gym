# Celeste Next Gym

Celeste 物理沙盒与技巧训练场。项目由 Rust 纯函数物理核心、WASM 桥、React 训练场和独立采集服务组成。

## 快速验证

```text
cargo test --workspace
node scripts/build-wasm.mjs
node scripts/e2e-real-collector.mjs
cd web && npm test && npm run test:wasm && npm run build
cd services/collector && npm test && npm run build
```

原版地图解析（游戏包解压后）：

```text
cargo run -p celeste-physics --example inspect_map -- vendor/celeste-game/Content/Maps/1-ForsakenCity.bin
```

生成并用真实 Everest 加载机制训练场：

```powershell
cargo run -p celeste-physics --example generate_playground -- mods\CelesteGymPlayground\Maps\CelesteGymPlayground\Playground.bin
$env:E2E_SHOW_WINDOWS = '1'
$env:E2E_AREA_SID = 'CelesteGymPlayground/Playground'
$env:E2E_ROOM = 'playground'
node scripts\e2e-real-collector.mjs
```

## 目录

- `crates/celeste-physics`：快照类型、Normal/Dash/Climb 物理子集、碰撞、BinaryPacker、C ABI。
- `crates/celeste-wasm`：MessagePack Web Worker/WASM 桥。
- `web`：WASM-only 逐帧训练界面、可编辑输入时间线、按需 state 缓存、录制回放与键位绑定。
- `services/collector`：Celeste/Everest 采集服务协议与可替换后端。
- `mods/CelesteGymCollector`：真实游戏逐帧输入注入和 126 字段快照采集 Mod。
- `mods/CelesteGymPlayground`：由 Rust 写入器生成的标准 Celeste `.bin` 机制训练场 Mod。
- `docs`：架构、原版源码审计和保真边界。

生成 WASM 需要先安装 `wasm32-unknown-unknown` target 和与依赖版本一致的 `wasm-bindgen-cli`。仓库同时包含已生成的浏览器资源；前端物理只运行 Rust WASM，加载失败会明确报错，不提供另一套模拟实现。

当前保真等级是 `source_informed_subset`，不是完整 1:1 实现。详见 `docs/architecture.md`。

`scripts/e2e-real-collector.mjs` 会构建并安装采集 Mod 和训练场 Mod、启动 Everest 与 HTTP 服务，并执行 Rust 差分。可通过 `E2E_AREA_ID` 选择原版区域，或通过 `E2E_AREA_SID` 加载自定义地图；现有真实场景的误差门槛为 `<= 0.01`。
