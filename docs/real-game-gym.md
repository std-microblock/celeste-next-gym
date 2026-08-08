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

对普通逐帧 RL，提交一个 input 即可。对 frame-skip 或技能控制器，可批量提交相同或预先规划的 input；`player_states` 保留实际执行的每个物理帧，而实体列表只抓取批次末尾一次，以避免 payload 随 `frames × entities` 膨胀。

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

2026-08-08 在 bundled Playground 的一次隐藏窗口 smoke 结果：reset 约 316 ms；16 帧 batch 约 270 ms；有效约 59.2 物理帧/秒；room grid 为 120×68，reset 时抓到 18 个运行时实体。这个数字是单个完整游戏进程的原生固定时间步表现，不代表多进程总吞吐。

## 当前限制

- 单个 Celeste 进程只有一个活动 episode；并行训练需要多个各自隔离端口/save/tmp 的游戏进程。
- 当前按原生固定时间步约 60 FPS 推进；batch step 只减少 HTTP/TCP 往返，不会跳过 Mod 更新。
- 不支持任意帧完整 clone/restore。`reset` 会重新创建 Level；跨死亡探索应保存动作轨迹、archive 或检查点重放信息。
- area/SID 必须已安装到该 Everest 实例。`/api/gym/reset` 不会动态热挂载请求体中的 `.bin`。
- `entities` 是批次末尾快照；若需要每帧实体轨迹，请用单帧 step，或由训练端根据多个短 batch 采样。
- 房间切换当前作为 episode 成功终止。跨房间 episode 需要在训练端对新 room 再 reset，或后续扩展终止策略。
