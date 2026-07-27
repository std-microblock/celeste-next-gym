# CelesteGymCollector

Everest 代码 Mod，为真实游戏 E2E 提供本地 TCP 采集端点。

- 监听 `127.0.0.1:32270`。
- 网络线程只负责 JSON 行协议；场景切换和玩家操作在游戏主线程执行。
- 支持 `ping`、`simulate_area` 和受认证的 `capture_start/status/stop/finalize`。
- 按帧替换 MoveX、MoveY、Jump、Dash、CrouchDash、Grab 输入。
- 返回起始状态及每帧结束后的状态，并反射采集 Player/Actor/Entity 的可序列化字段。
- 当前每帧稳定导出 126 个字段。
- `CELESTE_GYM_COLLECTOR_PORT` 可为隔离测试选择 Mod TCP 端口；默认仍为 `32270`。
- `CELESTE_GYM_RUN_NONCE` 会随 `ping` 一并返回，同时返回游戏进程 PID 和实际监听端口，供 runner 验证自己连接的是本次启动的子进程。
- `CELESTE_GYM_RECORDING_ROOT` 是 runner 创建的固定 per-run 录制根目录。协议不接受调用者提供输出路径；scenario、一次性 capture token 和所有派生路径都必须留在该物理目录内。

录制语义是 presentation frame，不是逐 Update frame。Mod 在每次真实 `Celeste.RenderCore` 完成后读取 backbuffer，将 viewport 确定性缩小为 320×180 BGRA，并记录当时最新的 E2E state index 与单调时间戳。固定时间步可能在一次 Draw 前推进多个 Update，也可能重复呈现同一个 state；manifest 会显式记录未呈现的 update 区间和重复呈现，逐 Update 精确性仍由 E2E trace 承担。

构建和真实 E2E：

```text
dotnet build mods/CelesteGymCollector/Source/CelesteGymCollector.csproj -c Release
node scripts/e2e-real-collector.mjs
```

测试脚本会把 Mod 安装进忽略提交的 `vendor/celeste-game/Mods/CelesteGymCollector`，原始游戏 zip 和 Everest 的 `orig` 备份不会被修改。

真实 E2E runner 只使用仓库自己的物理 vendor 安装，并为每次运行生成独立 save/tmp 目录和 manifest。不得按 `Celeste.exe` 进程名或人工观察到的 PID 清理游戏。
