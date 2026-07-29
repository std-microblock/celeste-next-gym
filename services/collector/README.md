# Celeste Collector Service

独立的 MessagePack HTTP 包装服务，用于把测试系统的模拟请求转交给 Celeste/Everest 采集后端。

真实 Celeste/Everest TCP 后端已经实现。默认启动仍返回 `NOT_CONFIGURED`；只有显式设置 `COLLECTOR_BACKEND=everest` 才连接游戏，避免把 mock 数据误当成真实数据。

## 运行

要求 Node.js 20 或更高版本。

```bash
npm install
npm run build
npm start
```

默认监听 `127.0.0.1:4318`。支持的环境变量：

| 变量                     | 默认值      | 说明                                              |
| ------------------------ | ----------- | ------------------------------------------------- |
| `COLLECTOR_HOST`         | `127.0.0.1` | 监听地址                                          |
| `COLLECTOR_PORT`         | `4318`      | 监听端口                                          |
| `COLLECTOR_TIMEOUT_MS`   | `30000`     | 单次后端采集超时                                  |
| `COLLECTOR_BACKEND`      | `none`      | `none` 或 `mock`                                  |
| `EVEREST_COLLECTOR_HOST` | `127.0.0.1` | Mod TCP 地址                                      |
| `EVEREST_COLLECTOR_PORT` | `32270`     | Mod TCP 端口                                      |
| `EVEREST_AREA_ID`        | `1`         | 当前真实采集使用的原版区域 ID                     |
| `EVEREST_AREA_SID`       | 未设置      | 自定义地图 SID；设置后优先按 SID 解析动态 Area ID |

仅测试协议时可显式使用：

```bash
COLLECTOR_BACKEND=mock npm start
```

mock 后端只复制初始快照并添加 `_collector_mock: true` 和 `_frame`，**不执行任何 Celeste 物理逻辑，不能作为对标数据**。

真实后端：

```text
COLLECTOR_BACKEND=everest npm start
```

完整自动 E2E 推荐从仓库根目录运行 `node scripts/e2e-real-collector.mjs`。

自动 runner 会动态选择 `EVEREST_COLLECTOR_PORT` 与 `COLLECTOR_PORT`，并先用 run nonce、Mod 回报 PID 和端口验证游戏子进程。手工运行时固定默认端口仍可用，但不得把未知的既有监听器当作真实测试后端。

可见窗口下对标第二章 `lvl_1` 的 DreamDash：

```powershell
$env:E2E_SHOW_WINDOWS = '1'
$env:E2E_AREA_ID = '2'
$env:E2E_ROOM = '1'
node scripts/e2e-real-collector.mjs
```

可见窗口下加载生成的机制训练场：

```powershell
$env:E2E_SHOW_WINDOWS = '1'
$env:E2E_AREA_SID = 'CelesteGymPlayground/Playground'
$env:E2E_ROOM = 'playground'
node scripts/e2e-real-collector.mjs
```

## API

### `GET /health`

返回 JSON 探活信息。服务未接入游戏时仍返回 HTTP 200（进程存活），但 `ready` 为 `false`：

```json
{
  "status": "ok",
  "ready": false,
  "backend": "not-configured",
  "detail": "..."
}
```

### `POST /api/simulate`

请求和响应均使用 `Content-Type: application/octet-stream`，body 为 MessagePack。

请求：

```text
{
  map: Uint8Array,                 // 非空原始 .bin 数据
  room?: string,                   // 可选房间名，例如 "1" 对应 lvl_1
  dream_dash?: boolean,            // Session.Inventory.DreamDash
  inputs: InputState[],
  initial_snapshot: Snapshot|null,
  frames: u32                      // 必须等于 inputs.length
}
```

成功响应（HTTP 200）：

```text
{ success: true, states: Snapshot[] } // states.length === frames + 1
```

错误响应同样是 MessagePack：

```text
{ success: false, code: "NOT_CONFIGURED", error: "..." }
```

主要状态码：

- `400 INVALID_REQUEST`：MessagePack 或字段校验失败
- `413 BODY_TOO_LARGE`：请求体超过限制
- `415 UNSUPPORTED_MEDIA_TYPE`：Content-Type 错误
- `502 BACKEND_ERROR`：后端失败或返回无效状态序列
- `503 NOT_CONFIGURED`：未配置真实采集后端
- `504 BACKEND_TIMEOUT`：采集超时

## Everest 后端

`src/everestBackend.ts` 已实现 `CollectorBackend`，将 HTTP MessagePack 请求转换为 Mod 的 JSON 行协议。

```ts
interface CollectorBackend {
  readonly name: string;
  collect(
    request: SimulateRequest,
    signal: AbortSignal,
  ): Promise<PlayerSnapshot[]>;
  health?(): Promise<{ ready: boolean; detail?: string }>;
  close?(): Promise<void>;
}
```

当前自动脚本负责游戏进程生命周期，Mod 可按原版 `area_id` 或自定义 `area_sid` 加载区域，逐帧注入输入并返回 `frames + 1` 个快照。训练场 `.bin` 会在启动前打包为 Everest Mod；请求体中的任意 `.bin` 动态热挂载仍是后续工作。

服务边界会再次校验返回数组长度及快照核心字段，因此不合规的 Mod 输出不会被当作成功数据返回。未知快照字段会保留，以便逐步扩展完整的 `Player.cs` 字段集合。

## 测试

```bash
npm test
npm run typecheck
```

测试不依赖 Celeste，覆盖 MessagePack 往返、未配置状态、Content-Type、畸形请求、字段校验、超时和无效后端输出。
