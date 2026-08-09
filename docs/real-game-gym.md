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

每个 actor 都有：

- 独立动态 Mod TCP port 和 HTTP port；
- 独立 `EVEREST_SAVEPATH` / `EVEREST_TMPDIR`；
- 独立 nonce，并要求 Mod handshake 同时匹配 nonce、精确 spawned Celeste PID 和 port；
- 单独的 immutable run manifest，以及 supervisor manifest；
- 默认隐藏窗口；需要调试时可加 `--show-windows`。

Ctrl+C/进程错误清理只会处理启动器记录的 child PID，并在终止前重新核对 executable path 和 creation time。无法证明 ownership 时会报警并保留进程，不会按进程名清理。`--smoke` 可用于 CI：所有 actor ready 后立即走同一 ownership-safe cleanup。

## 当前限制

- 单个 Celeste 进程只有一个活动 episode；并行训练需要多个各自隔离端口/save/tmp 的游戏进程。
- 高速模式以 batch 为调度单位；大量单帧 HTTP requests 仍可能接近外层 60 Hz latency。推荐把重复 action 或技能序列打成小 batch。
- `include_player_states=true` 会对每帧抓取完整反射状态并放大 JSON/MessagePack payload；追求吞吐时使用 `false`，按需用短 batch 或 `observe` 取样。
- 不支持任意帧完整 clone/restore。`reset` 会重新创建 Level；跨死亡探索应保存动作轨迹、archive 或检查点重放信息。
- area/SID 必须已安装到该 Everest 实例。`/api/gym/reset` 不会动态热挂载请求体中的 `.bin`。
- `entities` 是批次末尾快照；若需要每帧实体轨迹，请用单帧 step，或由训练端根据多个短 batch 采样。
- 房间切换当前作为 episode 成功终止。跨房间 episode 需要在训练端对新 room 再 reset，或后续扩展终止策略。
