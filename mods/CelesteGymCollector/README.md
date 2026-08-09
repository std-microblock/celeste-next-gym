# CelesteGymCollector

Everest 代码 Mod，为真实游戏 E2E 提供本地 TCP 采集端点。

- 默认优先监听 `127.0.0.1:32270`。若用户手工启动游戏且该默认端口被占用或被系统拒绝，Mod 会回退到 loopback 临时端口，并在日志与 `ping.collector_port` 中报告实际端口。
- 网络线程只负责 JSON 行协议；场景切换和玩家操作在游戏主线程执行。
- 支持 `ping`、`simulate_area`、持久 `gym_reset/step/observe/close`、受认证的 `capture_start/status/stop/finalize`，以及 `interactive_start/status/stop` 原生游玩逐 Update 录制。
- 按帧替换 MoveX、MoveY、Jump、Dash、CrouchDash、Grab 输入。
- 返回起始状态及每帧结束后的状态，并反射采集 Player/Actor/Entity 的可序列化字段。
- 当前每帧稳定导出 126 个字段。
- Gym reset 会创建全新的真实 `Level`，随后多个 step 批次在同一个 Level 和实体状态上继续。reset/observe 导出房间的无损 8px solid grid；step 返回逐帧玩家状态和批次末尾的连续坐标实体状态。实体 ID 仅在当前 episode 内稳定。
- `gym_reset.seed` 是可选的有符号 32 位整数。显式提供时会在创建 `Session` 或原地 `Level.Reload` 前重置 `Monocle.Calc.Random` 并清空其嵌套 RNG 栈；省略时保留游戏原有 RNG 行为。
- 原地 reset 会取消尚未结束的房间切换 coroutine，并清除 hit-stop、Dash Assist freeze 和暂停状态，避免新 Player 在 tracker 中存在但永远收不到 `Player.Update`。
- interactive 模式自动进入随仓库生成的 Playground；每次 `Player.Update` 前读取真实输入、整帧结束后抓取状态，停止时写出 `celeste-next-gym-trace` v1。它不替换 VirtualInput，也不把渲染帧当成物理帧。
- `CELESTE_GYM_COLLECTOR_PORT` 可为隔离测试和 direct actor 指定固定 Mod TCP 端口。显式配置的端口保持 fail-fast，绝不静默回退，绑定异常会明确包含 `127.0.0.1:<port>`；只有未设置环境变量时，默认 `32270` 才允许在 AccessDenied/AddressAlreadyInUse 后回退。
- `CELESTE_GYM_RUN_NONCE` 会随 `ping` 一并返回，同时返回游戏进程 PID 和实际监听端口，供 runner 验证自己连接的是本次启动的子进程。
- `CELESTE_GYM_RECORDING_ROOT` 是 runner 创建的固定 per-run 录制根目录。协议不接受调用者提供输出路径；scenario、一次性 capture token 和所有派生路径都必须留在该物理目录内。

录制语义是 presentation frame，不是逐 Update frame。Mod 在每次真实 `Celeste.RenderCore` 完成后读取 backbuffer，将 viewport 确定性缩小为 320×180 BGRA，并记录当时最新的 E2E state index 与单调时间戳。固定时间步可能在一次 Draw 前推进多个 Update，也可能重复呈现同一个 state；manifest 会显式记录未呈现的 update 区间和重复呈现，逐 Update 精确性仍由 E2E trace 承担。scenario 初始快照移动玩家时，镜头会先同步到原生 `Player.CameraTarget`；最终 state 首次呈现后还会继续采集 60 个 presentation frame，使 60 FPS 成片保留一秒收尾。

构建和真实 E2E：

```text
dotnet build mods/CelesteGymCollector/Source/CelesteGymCollector.csproj -c Release
node scripts/e2e-real-collector.mjs
node scripts/gym-actors.mjs --actors 1 --direct-tcp --seed-smoke --soak-room 2
node scripts/gym-actors.mjs --actors 1 --direct-tcp --input-lifecycle-smoke --input-lifecycle-rounds 100 --soak-room 2
```

测试脚本会把 Mod 安装进忽略提交的 `vendor/celeste-game/Mods/CelesteGymCollector`，原始游戏 zip 和 Everest 的 `orig` 备份不会被修改。

真实 E2E runner 只使用仓库自己的物理 vendor 安装，并为每次运行生成独立 save/tmp 目录和 manifest。不得按 `Celeste.exe` 进程名或人工观察到的 PID 清理游戏。

Gym 命令有意只允许一个活动 episode。`gym_step` 最多接受 4096 个输入帧，死亡、房间切换或 `max_episode_frames` 会提前结束批次。它不提供任意帧 clone/restore，也不绕过 Mod 的 C# 更新逻辑。
