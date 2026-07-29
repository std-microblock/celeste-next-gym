# 教程地图制作手册

本文说明 Web 训练模式的地图驱动格式、不可见 Trigger 的运行顺序、教程/Fuzz 脚本全部字段，以及一张教程地图从编写到验证的流程。当前可运行示例是 [`hyper-route.training.json`](../web/src/training/maps/hyper-route.training.json)。

## 1. 核心模型

训练的拥有者是**地图**，不是教程。目录中的一个 `TrainingVariant` 表示一张训练地图，它包含：

- `map`：可模拟的房间碰撞、实体和出生点；
- `initial`：玩家进入地图时的状态；
- `training`：一个 version 2 地图脚本；
- `training.modules[]`：任意数量的教程模块，每个模块各自绑定一个不可见 Trigger、一个教程和一个 Fuzz；
- `training.finish.trigger`：终点 Trigger。

一次模块的事件顺序如下：

1. 玩家碰到模块 Trigger，模块被“武装”，但 Fuzz 计时尚未开始。
2. 方向键可以用于走进 Trigger；只有新的动作输入（如 `jump`、`dash`、`crouch_dash`）或教程明确要求的方向组合才触发判定。
3. Trigger 后的第一个动作到来时，运行时以**动作发生前的真实玩家快照**执行 Fuzz；该动作是模块本地 `F0`。
4. 后续已声明 `verify: true` 的输入按 Fuzz 候选窗口逐个核对。
5. 失败沿用中断、慢放和失败面板；重试回到模块 R 点。
6. 成功不会中断地图。左上角增加一张包含 Objective 输出曲线、文字标注、精准度和反应帧数的 Timeline Toast；悬停任意采样点会显示点类型与速度等输出，玩家继续前进。
7. 完成所需模块后进入终点 Trigger，显示整图结算；模块 Toast 会动画汇入左侧记录，中间显示草莓与总成绩，右侧显示推荐地图。

## 2. Trigger 原理

Trigger 不加入 `GymMap.entities`，因此不会被物理层绘制、碰撞或序列化成 Celeste 实体。它只存在于 `training` JSON 中。

坐标与地图坐标一致：X 向右、Y 向下。`bounds` 是轴对齐矩形。运行时使用玩家底部中心 `SimState.pos` 构造碰撞框：站立时宽 8、高 11；下蹲时宽 8、高 6。矩形接触边界也算触发。

Trigger 采用“进入边沿”语义：玩家从区域外进入时触发一次；留在区域中不会每帧重复触发；离开后再次进入可重新触发尚未完成的模块。完成的模块本局不会再次武装。模块 Trigger 应使用全图唯一 `id`，也不能与终点 Trigger 的 `id` 重复。

设计建议：

- Trigger 覆盖动作起手前的一小段安全区域，不要覆盖整个房间。
- 不要让两个未完成模块的 Trigger 重叠。
- Trigger 后要留出足够空间，让 Fuzz 的 `observe_until` 能在撞墙或换房前结束。
- 玩家可以出生在 Trigger 内；此时地图加载即视为进入并武装该模块。
- `require_all_modules: true` 时，提前碰到终点只提示剩余模块数量；玩家离开再进入后可重新检查。

## 3. 最小地图脚本

```json
{
  "version": 2,
  "id": "my-training-map",
  "title": "我的训练图",
  "summary": "地图目录与结算推荐中显示的简介。",
  "modules": [
    {
      "id": "first-lesson",
      "trigger": {
        "id": "first-lesson-start",
        "bounds": { "x": 64, "y": 190, "width": 96, "height": 50 }
      },
      "end_trigger": {
        "id": "first-lesson-end",
        "bounds": { "x": 272, "y": 180, "width": 72, "height": 60 }
      },
      "tutorial": {
        "version": 2,
        "id": "first-lesson-tutorial",
        "title": "第一课",
        "summary": "本模块教授什么。",
        "entry": {
          "input_id": "dash_entry",
          "hint": "触发后下一个动作请冲刺。",
          "check": ["!current.dead", "current.state == state::dash"],
          "failure": { "title": "需要冲刺", "body": "请以冲刺开始。" }
        },
        "teaching": {
          "steps": [
            {
              "prompt": "按 Dash。",
              "order_error": {
                "title": "动作错误",
                "body": "第一个动作应为 Dash。"
              },
              "window_error": {
                "title": "时机错误",
                "body": "请在可行窗口内输入。"
              }
            }
          ]
        },
        "assist": {
          "result_sample_after_input_frames": 0,
          "auto_slowdown": {
            "enabled_by_default": true,
            "radius_frames": 12,
            "minimum_multiplier": 0.85
          }
        },
        "fuzz": {
          "version": 1,
          "inputs": [
            { "id": "dash_entry", "keys": ["dash"], "at": 0, "verify": true }
          ],
          "variables": [],
          "observe_until": 8,
          "success": ["!final.dead"],
          "objectives": [{ "type": "maximize", "expression": "final.speed.x" }],
          "search": {
            "bindings": {},
            "output": ["best", "windows", "coverage"]
          }
        }
      },
      "validation": {
        "initial_state": {
          "pos": { "x": 100, "y": 240 },
          "speed": { "x": 0, "y": 0 },
          "state": "Normal",
          "facing": true,
          "dashes": 1,
          "stamina": 110,
          "on_ground": true,
          "ducking": false,
          "can_dream_dash": true,
          "dead": false,
          "death_freeze_pending": false,
          "respawn_frames": 0,
          "dash_dir": { "x": 0, "y": 0 }
        }
      }
    }
  ],
  "finish": {
    "trigger": {
      "id": "map-finish",
      "bounds": { "x": 820, "y": 180, "width": 100, "height": 60 }
    },
    "require_all_modules": true
  }
}
```

## 4. 地图脚本字段总表

除明确标注“可选”的字段外均为必填。

### 根对象 `TrainingMapDocument`

| 字段                         | 类型               | 含义                                       |
| ---------------------------- | ------------------ | ------------------------------------------ |
| `version`                    | `2`                | 地图训练脚本版本。不是 Fuzz 的版本。       |
| `id`                         | string             | 地图脚本稳定 ID。存档、日志和 UI 使用。    |
| `title`                      | string             | 地图标题。                                 |
| `summary`                    | string             | 地图简介。                                 |
| `modules`                    | `TrainingModule[]` | 地图中的教程模块；允许多个。               |
| `finish`                     | object             | 终点配置。                                 |
| `finish.trigger`             | `TrainingTrigger`  | 进入后尝试打开结算的不可见终点区域。       |
| `finish.require_all_modules` | boolean            | `true` 时必须完成 `modules` 中的全部模块。 |

### `TrainingModule`

| 字段                       | 类型               | 含义                                                                                                 |
| -------------------------- | ------------------ | ---------------------------------------------------------------------------------------------------- |
| `id`                       | string             | 地图内唯一、稳定的模块 ID。                                                                          |
| `trigger`                  | `TrainingTrigger`  | 武装本模块的开始区域；编辑器中显示为蓝色。                                                             |
| `end_trigger`              | `TrainingTrigger`  | 完成本模块录制的结束区域；编辑器中显示为粉色。                                                         |
| `tutorial`                 | `TrainingDocument` | 本模块的提示、错误文案、辅助选项和 Fuzz。                                                            |
| `validation`               | object             | 离线训练校验专用数据，不参与玩家实际尝试。                                                           |
| `validation.initial_state` | `SimState`         | 校验此模块时的代表性起始快照，应放在 Trigger 附近并符合地图碰撞。                                    |
| `validation.fuzz`          | Fuzz object，可选  | 只用于离线校验的 Fuzz 覆盖；省略时使用 `tutorial.fuzz`。适合把实际技巧成功条件扩展为“最终越过障碍”。 |

编辑器的“录制当前区域”会从 `validation.initial_state` 开始，在首个非 WASD
动作按下时把该帧记为本地 F0，并在玩家进入 `end_trigger` 后自动回写
`tutorial.entry`、`teaching.steps`、`fuzz.inputs`、`observe_until`、`success` 和
`validation.initial_state`。“录制全部区域”只会按 `modules` 数组顺序武装开始区；WASD
仍然可以移动玩家，但默认不会写入生成的教程 JSON。

### `TrainingTrigger`

| 字段            | 类型   | 含义                  |
| --------------- | ------ | --------------------- |
| `id`            | string | 全图唯一 Trigger ID。 |
| `bounds.x`      | number | 左边界地图坐标。      |
| `bounds.y`      | number | 上边界地图坐标。      |
| `bounds.width`  | number | 宽度，必须为正数。    |
| `bounds.height` | number | 高度，必须为正数。    |

### `TrainingDocument`

| 字段       | 类型        | 含义                                                        |
| ---------- | ----------- | ----------------------------------------------------------- |
| `version`  | `2`         | 教程模块格式版本。                                          |
| `id`       | string      | 教程稳定 ID。可与模块 ID 不同。                             |
| `title`    | string      | Toast、失败面板和结算记录中显示的名称。                     |
| `summary`  | string      | 教程摘要。                                                  |
| `entry`    | object      | Trigger 后第一个动作的定义和前置检查。                      |
| `teaching` | object      | 每个受验证输入的逐步提示与错误文案。                        |
| `assist`   | object      | 教学辅助设置。                                              |
| `fuzz`     | Fuzz object | 输入搜索、成功条件和排序目标。Fuzz 内部格式仍为 version 1。 |

### `entry`

| 字段            | 类型       | 含义                                                                                                   |
| --------------- | ---------- | ------------------------------------------------------------------------------------------------------ |
| `input_id`      | string     | 必须指向 `fuzz.inputs` 中一个 `verify` 不为 `false`、且解析后位于 F0 的输入。它是 Trigger 后首个动作。 |
| `hint`          | string     | 模块刚武装时跟随玩家显示的提示。                                                                       |
| `check`         | `string[]` | 首动作模拟后的 `current` 快照必须全部满足的 Rhai 布尔表达式。由 Rust 执行，不在 JS 中求值。            |
| `failure.title` | string     | 首动作按键或 `check` 错误时的失败标题。                                                                |
| `failure.body`  | string     | 对应失败说明。                                                                                         |

### `teaching.steps[]`

步骤顺序应与从 `entry.input_id` 开始的 `verify: true` 输入顺序一致。

| 字段                 | 类型   | 含义                                       |
| -------------------- | ------ | ------------------------------------------ |
| `prompt`             | string | 当前等待此输入时显示的跟随提示。           |
| `order_error.title`  | string | 玩家按了其他动作时的失败标题。             |
| `order_error.body`   | string | 输入顺序错误说明。                         |
| `window_error.title` | string | 动作正确但不在任何可行候选帧时的失败标题。 |
| `window_error.body`  | string | 时机错误说明。                             |

### `assist`

| 字段                               | 类型    | 含义                                                                                                  |
| ---------------------------------- | ------- | ----------------------------------------------------------------------------------------------------- |
| `result_sample_after_input_frames` | number  | 为后续采样预留的帧数；当前成功 Toast 使用 Fuzz 返回的最终候选数据，字段保留给需要延迟展示结果的模块。 |
| `auto_slowdown.enabled_by_default` | boolean | 进入模块时是否默认启用自动慢放。                                                                      |
| `auto_slowdown.radius_frames`      | number  | 距离下一个最佳输入多少帧时开始渐进慢放；`<= 0` 表示不渐变。                                           |
| `auto_slowdown.minimum_multiplier` | number  | 作者期望的最低倍率。产品层另有限制，当前最多自动降低 30%，即实际不会低于基础倍率的 `0.7`。            |

## 5. Fuzz 字段总表

### Fuzz 根对象

| 字段            | 类型                    | 必填 | 含义                                                                 |
| --------------- | ----------------------- | ---- | -------------------------------------------------------------------- |
| `version`       | `1`                     | 是   | Celeste Fuzz 格式版本。                                              |
| `variables`     | array                   | 否   | 穷举变量；省略等价于空数组。                                         |
| `inputs`        | array                   | 否   | 输入声明；省略等价于空数组。教程至少需要一个受验证入口。             |
| `observe_until` | integer 或表达式 string | 是   | 从本地 F0 起模拟到哪个后置快照帧。必须非负，且不能早于最后一个输入。 |
| `success`       | `string[]`              | 否   | 最终快照需全部满足的表达式；省略为空。                               |
| `objectives`    | array                   | 否   | 对成功候选的稳定排序目标；先比较前面的目标。                         |
| `search`        | object                  | 否   | 变量固定值和输出模式。                                               |
| `limits`        | object                  | 否   | 搜索资源上限。所有设置值必须大于零。                                 |

### `variables[]`

| 字段         | 类型                    | 含义                                       |
| ------------ | ----------------------- | ------------------------------------------ |
| `name`       | string                  | 变量名；必须唯一，不能与保留上下文名冲突。 |
| `range.from` | integer 或表达式 string | 闭区间起点。可引用更早声明的变量。         |
| `range.to`   | integer 或表达式 string | 闭区间终点。可引用更早声明的变量。         |
| `range.step` | 正整数，可选            | 枚举步长，默认 `1`。                       |

### `inputs[]`

| 字段           | 类型                                          | 含义                                                                                                            |
| -------------- | --------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `id`           | string                                        | 前端教程使用的稳定输入 ID。Fuzz 引擎忽略额外的 `id`，但教程运行时需要它。                                       |
| `keys`         | `string[]`                                    | 同帧按键组合。合法值：`up`、`down`、`left`、`right`、`jump`、`dash`、`crouch_dash`、`grab`。不能为空或重复。    |
| `at`           | integer 或表达式 string                       | 按下帧。入口指向的输入解析后必须为 `0`。                                                                        |
| `held_time`    | integer、表达式 string 或 `"hold::inf"`，可选 | 方向保持帧数。任何含方向的输入都必须提供它；纯 `dash`/`crouch_dash` 不允许提供。方向+冲刺组合只对方向应用保持。 |
| `verify`       | boolean，可选                                 | 默认 `true`。`true` 表示玩家必须亲自命中；`false` 通常用于 F0 已保持方向，不占教学步骤。                        |
| `before_input` | string 或 `string[]`，可选                    | 输入应用前的快照条件，全部满足才保留候选。                                                                      |
| `after_input`  | string 或 `string[]`，可选                    | 输入应用后的快照条件，全部满足才保留候选。                                                                      |

### `objectives[]`

| 字段         | 类型       | 含义                                                                   |
| ------------ | ---------- | ---------------------------------------------------------------------- |
| `type`       | `maximize` | 让 `expression` 越大越优。                                             |
| `type`       | `minimize` | 让 `expression` 越小越优。                                             |
| `type`       | `approach` | 让 `expression` 尽量接近 `target`；此类型还必须提供有限数值 `target`。 |
| `expression` | string     | 返回数值的 Rhai 表达式。                                               |
| `target`     | number     | 仅 `approach` 使用的目标值。                                           |

### `search`

| 字段       | 类型                      | 含义                                                                                                                                                              |
| ---------- | ------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `bindings` | `Record<string, integer>` | 固定指定变量值，其余变量继续穷举。默认 `{}`。                                                                                                                     |
| `output`   | `string[]`                | 可用值：`best`、`windows`、`coverage`、`candidates`、`evaluations`、`top_N`（如 `top_10`）。训练桥会额外取得其运行时所需的全部候选与评估。默认 `best + windows`。 |

### `limits`

| 字段                        |      默认值 | 含义                         |
| --------------------------- | ----------: | ---------------------------- |
| `max_candidates`            |   1,000,000 | 最大候选组合数。             |
| `max_input_frames`          |         600 | 单候选最大输入/观察帧跨度。  |
| `max_trie_nodes`            |   5,000,000 | 前缀模拟缓存最大节点数。     |
| `max_cache_bytes`           | 536,870,912 | 前缀缓存估算字节上限。       |
| `max_expression_operations` |      10,000 | 单次 Rhai 表达式最大操作数。 |

## 6. 表达式可用字段

根据表达式位置，可使用以下上下文：

- `initial`：本候选 F0 动作前的快照；
- `before` / `after`：`before_input` / `after_input` 对应输入前后快照；
- `final`：`observe_until` 后的最终快照；
- `current`：教程 `entry.check` 使用的首动作后快照；
- 所有已声明变量名；
- 输入条件中还可用 `at`、`held_time`、`input_index`、`verify`。

快照公开字段：

| 字段                          | 类型           | 含义              |
| ----------------------------- | -------------- | ----------------- |
| `.pos.x` / `.pos.y`           | number         | 玩家位置。        |
| `.speed.x` / `.speed.y`       | number         | 玩家速度。        |
| `.state`                      | state enum     | 玩家状态。        |
| `.facing`                     | boolean        | 面向右为 `true`。 |
| `.dashes`                     | integer        | 当前冲刺数。      |
| `.stamina`                    | number         | 体力。            |
| `.on_ground`                  | boolean        | 是否着地。        |
| `.ducking`                    | boolean        | 是否下蹲。        |
| `.dead`                       | boolean        | 是否死亡。        |
| `.dash_dir.x` / `.dash_dir.y` | number         | 最近冲刺方向。    |
| `.last_aim.x` / `.last_aim.y` | number         | 最近瞄准方向。    |
| `.core_mode`                  | core-mode enum | Core 模式。       |

只注册了 `abs`、`min`、`max` 辅助函数；可以使用核心算术、比较和布尔运算，不能使用文件、网络、字符串/集合标准库。

状态常量为 `state::normal`、`climb`、`dash`、`swim`、`boost`、`red_dash`、`hit_squash`、`launch`、`pickup`、`dream_dash`、`summit_launch`、`dummy`、`frozen`、`reflection_fall`、`star_fly`、`temple_fall`、`cassette_fly`、`attract`、`intro_walk`、`intro_jump`、`intro_respawn`、`intro_wake_up`、`bird_dash_tutorial`、`intro_moon_jump`、`fling_bird`、`intro_think_for_a_bit`。Core 常量为 `core_mode::none`、`hot`、`cold`；无限保持为 `hold::inf`。

## 7. 地图与注册

JSON 只拥有训练逻辑；可模拟地图仍是 `GymMap`：

| 字段             | 含义                                                                                                             |
| ---------------- | ---------------------------------------------------------------------------------------------------------------- |
| `name` / `room`  | 地图显示名与房间 ID。                                                                                            |
| `bounds`         | 房间边界。                                                                                                       |
| `spawn`          | 默认出生点。                                                                                                     |
| `solids[]`       | 实体地形矩形。                                                                                                   |
| `entities[]`     | 物理实体；字段为 `kind`、`bounds`、`direction`、`name`，以及按实体需要提供的 `shielded`、`single_use`、`nodes`。 |
| `source_package` | 来源包；手写图通常为 `null`。                                                                                    |

在 technique 文件中导入 JSON 时同时添加 Node 所需的 import attribute：

```ts
import trainingJson from "../maps/my-map.training.json" with { type: "json" };
```

然后创建 `TrainingVariant`：`id`、`title`、`summary`、`map`、`initial`、`training`。目录展示的是地图；教程标题来自 `training.modules[].tutorial.title`。

`validation.initial_state` 至少应提供 `SimState` 的核心字段：`pos`、`speed`、`state`、`facing`、`dashes`、`stamina`、`on_ground`、`ducking`、`can_dream_dash`、`dead`、`death_freeze_pending`、`respawn_frames`、`dash_dir`。实体技巧还要提供对应运行时字段，例如 Booster 的 `boost_target`、`last_booster_target`、`booster_reuse_timer`、`state_timer`。完整类型以 [`model.ts`](../web/src/model.ts) 的 `SimState` 为准。

## 8. 结算指标

- 模块精准度：比较本次主 Objective 的实际 output 与该次 Fuzz 的最佳 output。公式为 `clamp(1 - abs(actual - best) / abs(best), 0, 1) * 100%`；最佳值为 0 时，仅实际值也为 0 才记 100%。例如主 Objective 是 `final.speed.x`，实际水平速度 300、Fuzz 最佳速度 325，则精准度约为 92.31%。输入早晚只作为 Timeline 的独立提示，不参与精准度。
- 反应速度：进入模块 Trigger 到首次动作 F0 之间的帧数；结算按 60 FPS 换算毫秒。
- 综合精准度：所有已完成模块精准度的算术平均。
- 总用时：地图全局帧数除以 60。
- 最佳精准度：本图所有模块的最高精准度。

成功 Toast 与最终结算左栏都会保留同一份 Timeline：Objective 曲线、成功窗口、Fuzz 最佳点和玩家输入均带文字标注；hover/focus 提示会组合显示“Fuzz 最佳点 / 你的输入 / 成功窗口 / 失败点 / 可行候选 / 未通过候选”等点类型，以及 `水平速度`、`垂直速度`或其他 Objective output。

## 9. 制作与验证清单

1. 在 `web/src/training/maps/` 新建 `*.training.json`，先画模块 Trigger 与终点 Trigger。
2. 为每个模块写教程入口、教学步骤与 Fuzz；保证 `entry.input_id` 指向 F0 的受验证输入。
3. 给每个模块准备可信的 `validation.initial_state`。
4. 在 technique 文件中建立/导入 `GymMap`，注册为一个目录地图。
5. 运行 `npm run test:training`：它会检查入口、Trigger ID，并用真实 `celeste-wasm` 确认每个模块至少有一个成功候选。
6. 运行 `npx vitest run` 验证 Trigger、session 与 UI 单测。
7. 运行 `npm run build` 做 TypeScript 与生产构建检查。
8. 手动从 Trigger 外走入，分别确认：错误首动作会失败、正确成功后不停止、多个模块均可触发、未完成时终点不结算、全部完成后结算三栏布局正常。

常见错误包括：把 Trigger 放进 `entities` 导致它参与物理层；入口 `at` 不是 0；方向输入遗漏 `held_time`；步骤数量/顺序和受验证输入不一致；`observe_until` 早于最后输入；校验快照与 Trigger 附近的真实状态差异过大；多个 Trigger 复用同一个 ID。
