# Interactive frame recorder

这个目录把“网页玩测试地图”“真实 Celeste/Everest 玩同一张地图”“逐物理帧记录”和“对比两份记录”放进同一套 trace 协议。地图源、Celeste `.bin`、参考 trace 和运行输出也都放在这里，不需要人工向游戏或网页补地图。

## 网页端游玩与记录

```text
cd web
npm ci
npm run dev
```

打开 Vite 给出的本地地址，点击“录制输入”后直接用键盘游玩。停在希望导出的末帧，点击“导出逐帧”；文件包含从 F0 到当前帧的输入、每帧状态和已解码地图。点击“对比逐帧”可选择另一份网页或游戏 trace，界面会报告首个不一致帧和位置/速度最大误差。

## 真实游戏游玩与记录

真实游戏启动器只允许在干净、非 detached HEAD 的 Git 主工作区运行，只使用仓库物理路径 `vendor/celeste-game`。正常使用应在 `master`；代理按仓库流程切到已提交的待测分支后也可运行。它会构建并安装 Collector、打包本目录附带的 Playground 地图、选择动态 loopback 端口、创建隔离的 save/tmp 和 run manifest，并用 nonce + 精确子进程 PID 完成握手。

```text
cd interactive-recorder
npm run game
```

启动器会直接进入 `CelesteGymPlayground/Playground` 的 `playground` 房间并开始记录。正常在游戏窗口游玩；结束时回到启动终端按 Enter。结果保存在 `interactive-recorder/recordings/<run>/`：

- `trace.json`：可直接交给网页或命令行比较器；
- `maps/CelesteGymPlayground/Playground.bin`：当次使用的精确游戏地图；
- `maps/playground.map.fixture.json`：可重新生成 `.bin` 的地图源镜像；
- `manifest.json`：地图 SHA-256、精确游戏进程身份、帧数和隔离 run manifest 路径。

录制帧边界与 E2E 一致：F0 是开始游玩前的玩家状态；每次真实 `Player.Update` 前采集原生输入，整次 Engine update 完成后采集状态。因此 `states.length === inputs.length + 1`。菜单、暂停和过场中没有发生玩家物理更新的渲染帧不会被伪装为模拟帧。

可用 `--max-frames` 限制最长录制，默认 36,000 帧：

```text
npm run game -- --max-frames=7200
```

## 对比与验证

```text
npm run compare -- recordings/<run>/trace.json examples/web-run-right.trace.json
npm run maps:check
npm test
npm run typecheck
```

比较器逐帧检查 position、speed、state、facing、dashes、stamina、grounded、ducking、death 九类字段。默认且最大容差均为 `0.01`；不一致时以非零退出码返回首个差异帧和字段。

`examples/` 内的右跑和右跳 trace 由当前 WASM 与附带 `.bin` 可复现生成：

```text
npm run samples
```

## Trace v1

所有文件使用 `celeste-next-gym-trace` v1：

```json
{
  "format": "celeste-next-gym-trace",
  "version": 1,
  "source": "web",
  "map": { "sid": "CelesteGymPlayground/Playground", "room": "playground", "binary": "maps/CelesteGymPlayground/Playground.bin" },
  "inputs": [],
  "states": []
}
```

网页 trace 在每帧 `snapshot` 中保留完整 WASM 状态；游戏 trace 在每帧 `fields` 中保留 Collector 反射得到的详细玩家字段。顶层九类字段是跨实现的稳定比较面。
