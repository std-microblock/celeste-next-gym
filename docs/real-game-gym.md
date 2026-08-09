# Real Celeste / Everest Gym backend

本接口让训练程序直接驱动 repository-owned Celeste + Everest，而不是 Rust 近似模拟器。游戏进程运行完整原版和已安装 Mod 代码；Collector Mod 在游戏主线程注入输入并抓取状态，Node collector 暴露 MessagePack HTTP API。

## 生命周期

```text
Celeste/Everest process
  └─ CelesteGymCollector TCP (JSON lines, loopback only)
       └─ services/collector HTTP (MessagePack, loopback only)
            └─ Python/Node trainer
```

一次 episode：

1. `POST /api/gym/reset` 加载 area/SID 和 room，返回玩家 state 0、8px 静态 solid grid、运行时实体。
2. 重复 `POST /api/gym/step`。每次可提交一帧或一个动作批次，游戏不会在批次之间重载。
3. 读取 `terminated/truncated/success`。死亡为失败终止，进入其他房间为成功终止，超过 `max_episode_frames` 为截断。
4. 用 `reset` 开始下一次尝试，或用 `close` 主动释放 episode。

## Python 最小客户端

依赖 `requests` 和 `msgpack`：

```python
import msgpack
import requests

BASE = "http://127.0.0.1:4318/api/gym"


def post(action: str, payload: dict) -> dict:
    response = requests.post(
        f"{BASE}/{action}",
        data=msgpack.packb(payload, use_bin_type=True),
        headers={"content-type": "application/octet-stream"},
        timeout=60,
    )
    body = msgpack.unpackb(response.content, raw=False)
    response.raise_for_status()
    if not body.get("success"):
        raise RuntimeError(body)
    return body


reset = post("reset", {
    "area_sid": "CelesteGymPlayground/Playground",
    "room": "playground",
    "skip_transitions": True,
    "max_episode_frames": 36_000,
    "include_entities": True,
    "include_player_states": False,
    "fast_mode": True,
})
observation = reset["observation"]
episode_id = observation["episode_id"]

neutral = {
    "move_x": 0,
    "move_y": 0,
    "jump_pressed": False,
    "jump_held": False,
    "dash_pressed": False,
    "crouch_dash_pressed": False,
    "grab_held": False,
    "talk_pressed": False,
}

transition = post("step", {
    "episode_id": episode_id,
    "inputs": [neutral] * 4,
})
observation = transition["observation"]
```

对普通逐帧 RL，提交一个 input 即可。对 frame-skip 或技能控制器，可批量提交相同或预先规划的 input。`gym_step.inputs` 支持 1～4096 帧。

- `fast_mode` 默认为 `false`。只有 reset 显式传入 `true` 的 episode 才 patch FNA 固定步长循环，并在隐藏窗口下禁止 Draw；普通游戏、`simulate_area`、录制 E2E 和默认 Gym 行为不变。
- `include_player_states` 默认为 `true`，会返回每一物理帧的完整玩家快照。长期 RL 通常应设为 `false`：每个 batch 只抓取并返回终态，避免 134 个反射字段随 batch 长度膨胀。
- `include_entities` 控制 batch 终态的实体反射快照；即使关闭，Celeste/Everest 仍会真实更新所有 Mod 实体，只是不序列化它们。

`observation.fast_mode` 会回显实际 episode 模式，训练端应检查它而不是假设加速已启用。

## 高速固定步长实现

加速不是跳过更新、修改 `Engine.DeltaTime`，也不是仅合并 HTTP/TCP。Collector 对 repository-owned FNA `Game.Tick` 做运行时 IL patch：

1. 在固定步长分支开始前，把一个已认证 Gym batch 映射为精确的 `N × TargetElapsedTime`；
2. 由 FNA 原始循环调用完整虚拟 `Update` N 次，因此 Celeste、Everest hook、Mod entity、StateMachine、Coroutine、Alarm 和 Tween 都按原本的 1/60 秒顺序执行；
3. 每帧仍由 Collector 主线程安装输入和检查死亡/换房；提前终止时清零剩余合成时间，不会在已返回 terminal state 后偷跑；
4. batch 结束后清理 FNA 的渲染 lag 标志，并在 `fast_mode` episode 中 `SuppressDraw`。

为保证长期 actor 稳定，单个外层 FNA Tick 最多执行 256 个 physics updates；1～4096 帧 API batch 会自动跨多个 outer Tick 完成，但仍只返回一个 Gym response。FMOD 和 AutoSplitter 属于 wall-clock 外部子系统：fast batch 中每个 outer Tick 最多更新一次，且不会为 synthetic physics frame 创建新的 FMOD event instance。游戏实体、StateMachine、Coroutine、碰撞和 Mod Update 仍逐 physics frame 执行。

同一 Area 的 `skip_transitions` reset 使用 fresh `Session` 加原生 `Level.Reload()`，重建 room gameplay state，同时复用 LevelLoader 创建的 renderer/backdrop/particle/graphics 基础设施。跨 Area reset 仍创建新的 `LevelLoader`。

合成物理时间不依赖 wall clock；wall clock 只决定请求何时进入下一次主线程 Tick。单帧 request 仍会受到一次外层 Tick/IPC 延迟，训练时应使用合适的 action repeat 或技能 batch。

## 观测坐标

- 玩家和实体位置均为 Celeste 世界像素坐标，可包含非 8px 对齐和亚像素值。
- `room_geometry.bounds` 为当前房间世界像素矩形。
- `room_geometry.tile_origin` 为房间左上角的世界 tile 坐标。
- `room_geometry.solids[y][x] == "1"` 表示该 8px tile 在原生 `Level.SolidsData` 中为 solid。
- 非 tile 碰撞体（JumpThru、Spikes、移动平台和 Mod 实体）通过 `entities` 返回，不会被强行栅格化。
- `EntityFrame.collider = [left, top, width, height]` 是相对实体 Position 的 Collider 包围几何。

实体的 `type` 使用运行时 CLR 完整类型名，所以新 Mod 实体无需先加入固定词表。`fields` 是有限制的反射快照：primitive、enum、`Vector2`、`Rectangle` 以及小型数组/列表会保留，复杂引用对象会忽略。训练端应把 `type + position + collider + fields` 视为本图可学习的对象状态，而不是跨 Mod 稳定 ABI。

## 启动和真实验证

collector service 不负责猜测或清理任意 Celeste 进程。生产训练启动器应复用 `scripts/e2e-real/isolation` 的约束：

- 只使用物理路径 `vendor/celeste-game`；
- 每个进程使用隔离的 `EVEREST_SAVEPATH` 和 `EVEREST_TMPDIR`；
- 动态分配 loopback 端口；
- 用随机 run nonce、Mod 回报 PID 和启动器实际 child PID 完成 handshake；
- 只清理启动器自己创建且 executable path/creation time 仍匹配的 child。

仓库 smoke 命令：

```powershell
npm --prefix services/collector run build
$env:E2E_GYM_SMOKE = '1'
$env:E2E_COLLECT_ONLY = '1'
node scripts/e2e-real-collector.mjs --target playground --scenario playground-load
```

高速确定性和吞吐 gate：

```powershell
$env:E2E_GYM_FAST_SMOKE = '1'
$env:E2E_COLLECT_ONLY = '1'
node scripts/e2e-real-collector.mjs --target playground --scenario playground-load
```

该 gate 先用普通 loop 与 fast loop 各执行同一段 120 帧输入，并以最多 `0.01` 的 tolerance 比较 position、speed、state、facing、dashes、stamina、grounded、ducking 和 death；随后要求 1024 帧 batch 实测严格超过 60 physics FPS。

2026-08-09 在 bundled Playground、隐藏窗口、RTX 4070 Ti SUPER 主机的实测：普通 loop 59.8 physics FPS；fast 120 帧为 6640.7 physics FPS；fast 1024 帧为 20365.9 physics FPS；普通/高速九类玩家状态一致。具体吞吐随 Room、Mod 和实体量变化，这不是对任意 Mod 的固定保证。

## 长期训练 actor

下面的命令不会在 smoke 完成后退出。它会构建并安装 Collector，启动 N 个隔离 Celeste + HTTP collector，输出每个 actor 的 Gym URL 和 manifest，然后一直运行到 Ctrl+C：

```powershell
node scripts/gym-actors.mjs --actors 4 --area-id 1
```

自定义已安装 Mod SID：

```powershell
node scripts/gym-actors.mjs --actors 2 --area-id 0 --area-sid MyMod/MyMap
```

训练器不需要 HTTP facade 时，可直接启动 Mod TCP actors：

```powershell
node scripts/gym-actors.mjs --actors 4 --area-id 1 --direct-tcp
```

Supervisor manifest 会为每个 slot 输出稳定的 `tcp_endpoint` / `tcp_host` / `tcp_port`，以及当前 generation 的 `auth.run_nonce` 和 `auth.process_id`。Generation restart 会复用相同 TCP port，但 nonce 和 PID 必然更新；客户端必须重新读取 supervisor manifest，并在恢复 rollout 前发送 `ping` 验证三元组 `run_nonce + process_id + collector_port`。

Direct protocol 是 UTF-8 newline-delimited JSON：连接 loopback port，发送一个 JSON object 加 `\n`，读取对应的 `\n` 结尾 JSON response。Connection 可按顺序复用任意多次，从而避免每个短 action batch 的 TCP handshake；仍兼容发送一次后立即关闭的旧客户端。同一 connection 当前只允许一个 in-flight request，request/response 顺序严格一致。Gym 命令为 `gym_reset`、`gym_step`、`gym_observe`、`gym_close`；每个命令都必须携带当前 generation 的 `run_nonce` 和 `process_id`。`ping` 不需要认证，并返回用于 ownership handshake 的 nonce、PID 和 port。

2026-08-09 的 4-actor persistent-connection gate 在每个 actor 复用单一 socket，完成 12 episodes、1,237 个短 action batches 和 5,526 physics frames，聚合 869.9 physics FPS，所有 slot 的 `restart_count` 均为 0。另一个 forced-generation gate 验证 TCP port 保持不变，而 nonce 和 PID 同时轮换。

每个 actor 都有：

- 独立动态 Mod TCP port 和 HTTP port；
- 独立 `EVEREST_SAVEPATH` / `EVEREST_TMPDIR`；
- 独立 nonce，并要求 Mod handshake 同时匹配 nonce、精确 spawned Celeste PID 和 port；
- 单独的 immutable run manifest，以及 supervisor manifest；
- 默认隐藏窗口；需要调试时可加 `--show-windows`。

Ctrl+C/进程错误清理只会处理启动器记录的 child PID，并在终止前重新核对 executable path 和 creation time。无法证明 ownership 时会报警并保留进程，不会按进程名清理。`--smoke` 可用于 CI：所有 actor ready 后立即走同一 ownership-safe cleanup。

每个 generation 的 Celeste/collector stdout、stderr、exit code 和 signal 都写入 actor manifest。若 child 异常退出，supervisor 会先验证并清理旧 generation，等待原 Mod/HTTP ports 确实释放，再以新 nonce、PID 和隔离 save/tmp 重启；**同一 actor slot 的公开 `gym_url` 和两个 port 不变**。Supervisor manifest 记录当前 `generation`、`restart_count` 和最新 generation manifest。

长期压力 gate：

```powershell
node scripts/gym-actors.mjs `
  --actors 1 --area-id 1 `
  --soak-resets 1000 --soak-room 2 --soak-frames 1536
```

Neutral reset gate 之外，还应运行 seeded policy-action gate。它用短 batch 确定性覆盖左右移动、跳跃按住/释放、dash、crouch dash、grab、死亡后的 fresh reset，以及可能的 room transition；任一 child exit 都会立即失败，不会用 supervisor restart 掩盖问题：

```powershell
node scripts/gym-actors.mjs `
  --actors 1 --area-id 1 `
  --soak-policy --soak-seed 20260809 --soak-action-frames 8 `
  --soak-resets 20 --soak-room 2 --soak-frames 1536
```

2026-08-09 的 repository-owned Celeste 实测完成 1,000 次 room 2 fresh reset、1,536,000 physics frames，耗时 141.0 秒，有效 10,891.8 physics FPS；观测实体数稳定为 45，最大 47。另一个 50-reset gate 在第 25 次强制 generation restart：重启前后公开 URL 均为 `http://127.0.0.1:59570/api/gym`，generation 从 0 变为 1，随后继续完成剩余 reset。

## 当前限制

- 单个 Celeste 进程只有一个活动 episode；并行训练需要多个各自隔离端口/save/tmp 的游戏进程。
- 同一个 actor URL 不能同时交给训练器和评估器；第二个客户端的 `reset` 会使第一个客户端的 episode id 失效。并发评估应使用另一个 actor slot。
- 高速模式以 batch 为调度单位；大量单帧 HTTP requests 仍可能接近外层 60 Hz latency。推荐把重复 action 或技能序列打成小 batch。
- Fast mode 有意禁用 synthetic physics frame 的音频 event 创建；依赖 FMOD playback position 作为 gameplay state 的非标准 Mod 需要单独适配，不能把声音播放状态当作 fast Gym ABI。
- Gym episode 活跃期间会禁用 controller rumble。Dash、死亡和攀爬等动作会高频调用 `GamePad.SetVibration`；headless actor 不消费震动反馈，而且无手柄/隐藏窗口下的原生 vibration backend 不适合作为训练所需的 gameplay side effect。
- Gym episode 活跃期间会禁用 `AutoSplitterInfo.Update`。Autosplitter 会遍历全局 SaveData/AreaKey，而 Gym reset 正在原地替换 fresh Session；headless 训练不使用分段计时，隔离该 observer 可避免加载窗口读取到不一致的 level-set 状态。
- 同 Area in-place reset 遵循 Celeste `Level.Reload()` 的 Global-tag 生命周期。把关键关卡状态藏在跨死亡 Global entity、且不从 fresh Session 恢复的 Mod，应提供自己的 reset hook 或选择进程 generation restart。
- `include_player_states=true` 会对每帧抓取完整反射状态并放大 JSON/MessagePack payload；追求吞吐时使用 `false`，按需用短 batch 或 `observe` 取样。
- 不支持任意帧完整 clone/restore。`reset` 会重新创建 Level；跨死亡探索应保存动作轨迹、archive 或检查点重放信息。
- area/SID 必须已安装到该 Everest 实例。`/api/gym/reset` 不会动态热挂载请求体中的 `.bin`。
- `entities` 是批次末尾快照；若需要每帧实体轨迹，请用单帧 step，或由训练端根据多个短 batch 采样。
- 房间切换当前作为 episode 成功终止。跨房间 episode 需要在训练端对新 room 再 reset，或后续扩展终止策略。
