# Celeste 原版源码移植审计

本文为 Rust 物理核心和 `.bin` 地图解析器提供可追溯的实现依据。结论来自仓库内的 Celeste 1.4.0.0 FNA C# 源码，不把需求书、Wiki 或二手常数表当作原版行为的证据。

## 1. 审计对象与版本锚点

审计的主要文件：

- `vendor/celeste-fna/Celeste/Player.cs`
- `vendor/celeste-fna/Celeste/BinaryPacker.cs`
- `vendor/celeste-fna/Celeste/RunLengthEncoding.cs`
- `vendor/celeste-fna/Celeste/MapData.cs`
- `vendor/celeste-fna/Celeste/LevelData.cs`
- `vendor/celeste-fna/Celeste/EntityData.cs`

为解释状态机和碰撞调用顺序，另核对：

- `vendor/celeste-fna/Celeste/Actor.cs`
- `vendor/celeste-fna/Monocle/StateMachine.cs`
- `vendor/celeste-fna/Monocle/Coroutine.cs`
- `vendor/celeste-fna/Monocle/Engine.cs`

`Celeste` 构造器明确声明版本 `1.4.0.0` 并启用固定时间步，见 `Celeste/Celeste.cs:62-68`。文件行号以当前工作区副本为准。为防止后续替换源码导致行号静默漂移，主要文件 SHA-256 如下：

| 文件 | SHA-256 |
|---|---|
| `Player.cs` | `2f8e5c7151ad6046a581810dbaa638f7b7b6756da0567538de2d2c2ff6d3a589` |
| `BinaryPacker.cs` | `1520ef04f8460b7629f549357d6e01468f05938da6feb5c938e50383b5d5c244` |
| `RunLengthEncoding.cs` | `82b86a974450138adb9e5980c80713004e995b778b288046236f50b4c2cd9b4c` |
| `MapData.cs` | `24dbb90c298fccdbe1b0be29ff305fd34d77d12e5929fa6509c97b640820cfd3` |
| `LevelData.cs` | `7ea4a9c89bd511655d7ae2d24e67d45dcb1074f154b059ca81d4436225837278` |
| `EntityData.cs` | `7a7a3fb4bea0935baa1e333940ac067126edc2220e4e9ea1b542c89d8ffa8a5b` |

## 2. 首要结论

1. `Player.cs` 不是一个可直接逐函数调用的独立物理模块。每帧先在 `Player.Update` 中更新接地、输入派生值和多个计时器，随后 `base.Update()` 驱动组件中的状态机/协程，最后才执行水平、垂直位移和碰撞回调。关键顺序见 `Player.cs:1425-1778`、`Player.cs:1787-1805`、`Entity.cs:474-477`、`StateMachine.cs:168-182`。
2. 位置不是简单的 `pos += speed * dt`。`Actor` 保存子像素 `movementCounter`，以 `MidpointRounding.ToEven` 舍入，再逐像素移动；撞墙时清零相应轴的余数。见 `Actor.cs:14-30`、`Actor.cs:186-208`、`Actor.cs:210-289`。
3. Dash 是状态更新与协程共同实现的。进入 Dash 的当帧 `DashBegin` 先把速度清零，`DashCoroutine` 首句 `yield return null`，下一次协程推进才设置 dash 速度。不能把它简化成“按键当帧立刻赋值 240”。见 `Player.cs:4276-4314`、`Player.cs:4465-4489`，以及协程推进规则 `Coroutine.cs:35-81`。
4. 需求书中的 `PlayerState` 只列到 `Attract = 22`，但 1.4.0.0 原码实际还有状态 23、24、25。若快照枚举缺失三项，不能称为完整状态机兼容。见 `Player.cs:349-399`。
5. `.bin` 是通用树形容器，不是“碰撞网格的固定结构体”。`BinaryPacker` 只恢复带类型属性的元素树；`MapData`/`LevelData` 再按元素名解释 levels、tiles、entities、triggers 等。见 `BinaryPacker.cs:270-343`、`MapData.cs:107-191`、`LevelData.cs:113-388`。

## 3. 每帧执行模型

### 3.1 `Player.Update` 的物理相关顺序

下列顺序会影响离散帧结果，Rust 实现应显式保留：

1. 保存 `PreviousPosition`，清 `climbTriggerDir`，更新辅助模式和若干非核心计时器，见 `Player.cs:1425-1498`。
2. 基于当前状态、当前 `Speed.Y` 和向下 1 像素碰撞计算 `onGround`；DreamDash 强制不接地，Swim 强制安全地面，见 `Player.cs:1499-1541`。
3. 若上一帧存在 wall slide，递减 `wallSlideTimer`，并在本帧状态更新前把 `wallSlideDir` 清零，见 `Player.cs:1555-1559`。
4. 处理 wall boost 窗口、落地回满体力、dash attack/cooldown/refill cooldown、土狼时间、可变跳计时、强制水平输入、无风窗口等，见 `Player.cs:1560-1656`。
5. 更新 `moveX`、朝向、墙速保留和 climb-hop 延迟水平速度，见 `Player.cs:1632-1695`。
6. 调用 `base.Update()`，按组件插入顺序执行 `StateMachine.Update`；状态 update 返回目标状态，状态切换同步调用旧状态 end、新状态 begin，并替换协程，随后同一状态机更新中推进协程一次。见 `Player.cs:1778`、`StateMachine.cs:28-75`、`StateMachine.cs:168-182`。
7. 状态更新完成后执行 JumpThru 向上辅助、水平 dash 落地 snap，然后依次 `MoveH`、`MoveV`，见 `Player.cs:1787-1805`。
8. 位移之后才处理进出水、触发器、边界等，见 `Player.cs:1807-1918`。

### 3.2 时间步与浮点

- `Engine.DeltaTime = RawDeltaTime * TimeRate * TimeRateB`，见 `Engine.cs:224-228`。普通固定帧模拟通常使用 `f32(1/60)`，但 Freeze、Dash Assist、TimeRate 改变会使“每次游戏循环都等于 1/60 秒”的假设失效。
- 原码的物理量为 C# `float`/XNA `Vector2`，Rust 应在每个中间步骤维持 `f32`，不能用 `f64` 累积后只在输出时截断。
- `Actor.MoveH/MoveV` 的 bankers rounding（ties-to-even）和撞击时余数清零是可观测状态；`PlayerSnapshot` 若不包含两个轴的 movement remainder，同一 `(pos, speed)` 不足以保证下一帧相同。见 `Actor.cs:14-30`、`Actor.cs:186-220`、`Actor.cs:238-269`。

## 4. Player 核心常数

### 4.1 Normal / 跑跳 / 下落

| 常数 | 值 | 原码位置 | 用途 |
|---|---:|---|---|
| `MaxFall` | `160` | `Player.cs:125` | 普通最大下落速度 |
| `Gravity` | `900` | `Player.cs:127` | 垂直加速度 |
| `HalfGravThreshold` | `40` | `Player.cs:129` | 跳键按住且 `abs(speedY) < 40` 时重力乘 0.5 |
| `FastMaxFall` | `240` | `Player.cs:131` | 按下时快速下落上限 |
| `FastMaxAccel` | `300` | `Player.cs:133` | `maxFall` 接近目标的速率 |
| `MaxRun` | `90` | `Player.cs:135` | 普通水平目标速度 |
| `RunAccel` | `1000` | `Player.cs:137` | 向输入目标加速 |
| `RunReduce` | `400` | `Player.cs:139` | 已超速且同向输入时的减速 |
| `AirMult` | `0.65` | `Player.cs:141` | 空中水平加/减速倍率 |
| `HoldingMaxRun` | `70` | `Player.cs:143` | 携带慢跑物体时目标速度 |
| `DuckFriction` | `500` | `Player.cs:149` | 地面蹲伏水平摩擦 |
| `JumpGraceTime` | `0.1 s` | `Player.cs:161` | 土狼时间 |
| `JumpSpeed` | `-105` | `Player.cs:163` | 普通跳跃纵向速度 |
| `JumpHBoost` | `40` | `Player.cs:165` | 跳跃时按输入方向追加水平速度 |
| `VarJumpTime` | `0.2 s` | `Player.cs:167` | 可变跳窗口 |
| `CeilingVarJumpGrace` | `0.05 s` | `Player.cs:169` | 撞顶后仅在剩余窗口 `< 0.15` 时清零；等价保留前 0.05 s |
| `UpwardCornerCorrection` | `4 px` | `Player.cs:171` | 普通向上撞角水平修正范围 |
| `DashingUpwardCornerCorrection` | `5 px` | `Player.cs:173` | 竖直 dash 向上撞角修正范围 |
| `WallSpeedRetentionTime` | `0.06 s` | `Player.cs:175` | 撞墙水平速度保留窗口 |
| `WallJumpCheckDist` | `3 px` | `Player.cs:177` | 普通墙跳探测距离 |
| `SuperWallJumpCheckDist` | `5 px` | `Player.cs:179` | 向上 dash 时的墙跳探测距离 |
| `WallJumpForceTime` | `0.16 s` | `Player.cs:181` | 墙跳强制水平输入时间 |
| `WallJumpHSpeed` | `130` | `Player.cs:183` | 墙跳水平速度 |
| `WallSlideStartMax` | `20` | `Player.cs:185` | wall slide 初始下落目标 |
| `WallSlideTime` | `1.2 s` | `Player.cs:187` | wall slide 由 20 插值回 160 的总时长 |

### 4.2 Super / Dash

| 常数 | 值 | 原码位置 | 用途 |
|---|---:|---|---|
| `SuperJumpSpeed` | `-105` | `Player.cs:197` | super 纵向速度 |
| `SuperJumpH` | `260` | `Player.cs:199` | super 水平速度 |
| `DuckSuperJumpXMult` | `1.25` | `Player.cs:157` | crouched super/hyper 水平倍率 |
| `DuckSuperJumpYMult` | `0.5` | `Player.cs:159` | crouched super/hyper 纵向倍率 |
| `SuperWallJumpSpeed` | `-160` | `Player.cs:201` | wallbounce 纵向速度 |
| `SuperWallJumpVarTime` | `0.25 s` | `Player.cs:203` | wallbounce 可变跳窗口 |
| `SuperWallJumpForceTime` | `0.2 s` | `Player.cs:205` | 常数存在，但当前 `SuperWallJump` 实现没有设置 `forceMoveXTimer` |
| `SuperWallJumpH` | `170` | `Player.cs:207` | wallbounce 水平速度 |
| `DashSpeed` | `240` | `Player.cs:209` | dash 初速度标量 |
| `EndDashSpeed` | `160` | `Player.cs:211` | dash 正常结束速度标量 |
| `EndDashUpMult` | `0.75` | `Player.cs:213` | dash 结束时上升速度倍率 |
| `DashTime` | `0.15 s` | `Player.cs:215` | 普通 dash 协程等待时间 |
| `SuperDashTime` | `0.3 s` | `Player.cs:217` | SuperDashing assist 的持续时间 |
| `DashCooldown` | `0.2 s` | `Player.cs:219` | 再次开始 dash 的冷却 |
| `DashRefillCooldown` | `0.1 s` | `Player.cs:221` | 接地/水中补 dash 前的冷却 |
| `DashHJumpThruNudge` | `6 px` | `Player.cs:223` | 水平 dash 与 JumpThru 垂直贴合范围 |
| `DashCornerCorrection` | `4 px` | `Player.cs:225` | dash 水平/落地边角修正范围 |
| `DashVFloorSnapDist` | `3 px` | `Player.cs:227` | 水平 dash 向下贴地距离 |
| `DashAttackTime` | `0.3 s` | `Player.cs:229` | 攻击判定可延续到 dash 状态结束后 |
| `DodgeSlideSpeedMult` | `1.2` | `Player.cs:155` | 斜下 dash 落地转水平后的速度倍率 |

### 4.3 Climb

| 常数 | 值 | 原码位置 | 用途 |
|---|---:|---|---|
| `ClimbMaxStamina` | `110` | `Player.cs:253` | 最大体力/落地恢复值 |
| `ClimbUpCost` | `45.4545441 /s` | `Player.cs:255` | 向上攀爬消耗 |
| `ClimbStillCost` | `10 /s` | `Player.cs:257` | 空中静止抓墙消耗 |
| `ClimbJumpCost` | `27.5` | `Player.cs:259` | 离地 climb jump 即时消耗；wall boost 可返还 |
| `ClimbCheckDist` | `2 px` | `Player.cs:261` | 抓墙探测距离 |
| `ClimbUpCheckDist` | `2 px` | `Player.cs:263` | 抓墙向上修正范围 |
| `ClimbNoMoveTime` | `0.1 s` | `Player.cs:265` | 刚抓墙时禁止主动上下移动/体力消耗 |
| `ClimbTiredThreshold` | `20` | `Player.cs:267` | tired 判定阈值为严格 `< 20`，UI danger 常用 `<= 20` |
| `ClimbUpSpeed` | `-45` | `Player.cs:269` | 上爬目标速度 |
| `ClimbDownSpeed` | `80` | `Player.cs:271` | 下爬目标速度 |
| `ClimbSlipSpeed` | `30` | `Player.cs:273` | 顶缘滑落目标速度 |
| `ClimbAccel` | `900` | `Player.cs:275` | 攀爬纵向接近速率 |
| `ClimbGrabYMult` | `0.2` | `Player.cs:277` | 进入攀爬时保留纵速比例 |
| `ClimbHopY` | `-120` | `Player.cs:279` | 爬上边缘纵向速度上限 |
| `ClimbHopX` | `100` | `Player.cs:281` | 爬上边缘水平速度 |
| `ClimbHopForceTime` | `0.2 s` | `Player.cs:283` | climb hop 强制 `moveX = 0` 时间 |
| `ClimbJumpBoostTime` | `0.2 s` | `Player.cs:285` | climb jump 的 wall boost 返还窗口 |
| `ClimbHopNoWindTime` | `0.3 s` | `Player.cs:287` | climb hop 后无风窗口 |

### 4.4 碰撞体与必须进入快照的隐状态

- 站立 collider：`8 x 11`, offset `(-4, -11)`；蹲伏 collider：`8 x 6`, offset `(-4, -6)`，见 `Player.cs:605-607`。
- 伤害判定框不同：站立 `8 x 9`, offset `(-4, -11)`；蹲伏 `8 x 4`, offset `(-4, -6)`，见 `Player.cs:609-611`。
- 除需求书已列字段外，至少还需：两个轴 movement remainder、`AutoJump/AutoJumpTimer`、`forceMoveX`、`wallSpeedRetained`、`wallBoostDir`、`maxFall`、`dashStartedOnGround`、`lastClimbMove`、`hopWaitX/hopWaitXSpeed`、`climbHopSolid` 的运动关联、`beforeDashSpeed`、`demoDashed`、`canCurveDash`、`wasOnGround`、`LiftSpeed/lastLiftSpeed` 及其计时器。相关字段见 `Player.cs:453-603`、`Player.cs:691-695` 和 `Actor.cs:14-30`。

## 5. 状态编号与回调映射

构造器创建 26 状态的 `StateMachine`，映射见 `Player.cs:1145-1171`。参数顺序是 update、coroutine、begin、end，定义见 `StateMachine.cs:152-158`。

| ID | 常量 | Update | Coroutine | Begin | End |
|---:|---|---|---|---|---|
| 0 | `StNormal` | `NormalUpdate` | — | `NormalBegin` | `NormalEnd` |
| 1 | `StClimb` | `ClimbUpdate` | — | `ClimbBegin` | `ClimbEnd` |
| 2 | `StDash` | `DashUpdate` | `DashCoroutine` | `DashBegin` | `DashEnd` |
| 3 | `StSwim` | `SwimUpdate` | — | `SwimBegin` | — |
| 4 | `StBoost` | `BoostUpdate` | `BoostCoroutine` | `BoostBegin` | `BoostEnd` |
| 5 | `StRedDash` | `RedDashUpdate` | `RedDashCoroutine` | `RedDashBegin` | `RedDashEnd` |
| 6 | `StHitSquash` | `HitSquashUpdate` | — | `HitSquashBegin` | — |
| 7 | `StLaunch` | `LaunchUpdate` | — | `LaunchBegin` | — |
| 8 | `StPickup` | — | `PickupCoroutine` | — | — |
| 9 | `StDreamDash` | `DreamDashUpdate` | — | `DreamDashBegin` | `DreamDashEnd` |
| 10 | `StSummitLaunch` | `SummitLaunchUpdate` | — | `SummitLaunchBegin` | — |
| 11 | `StDummy` | `DummyUpdate` | — | `DummyBegin` | — |
| 12 | `StIntroWalk` | — | `IntroWalkCoroutine` | — | — |
| 13 | `StIntroJump` | — | `IntroJumpCoroutine` | — | — |
| 14 | `StIntroRespawn` | — | — | `IntroRespawnBegin` | `IntroRespawnEnd` |
| 15 | `StIntroWakeUp` | — | `IntroWakeUpCoroutine` | — | — |
| 16 | `StBirdDashTutorial` | `BirdDashTutorialUpdate` | `BirdDashTutorialCoroutine` | `BirdDashTutorialBegin` | — |
| 17 | `StFrozen` | `FrozenUpdate` | — | — | — |
| 18 | `StReflectionFall` | `ReflectionFallUpdate` | `ReflectionFallCoroutine` | `ReflectionFallBegin` | `ReflectionFallEnd` |
| 19 | `StStarFly` | `StarFlyUpdate` | `StarFlyCoroutine` | `StarFlyBegin` | `StarFlyEnd` |
| 20 | `StTempleFall` | `TempleFallUpdate` | `TempleFallCoroutine` | — | — |
| 21 | `StCassetteFly` | `CassetteFlyUpdate` | `CassetteFlyCoroutine` | `CassetteFlyBegin` | `CassetteFlyEnd` |
| 22 | `StAttract` | `AttractUpdate` | — | `AttractBegin` | `AttractEnd` |
| 23 | `StIntroMoonJump` | — | `IntroMoonJumpCoroutine` | — | — |
| 24 | `StFlingBird` | `FlingBirdUpdate` | `FlingBirdCoroutine` | `FlingBirdBegin` | `FlingBirdEnd` |
| 25 | `StIntroThinkForABit` | — | `IntroThinkForABitCoroutine` | — | — |

状态切换不是单纯改整数：setter 依次记录 `PreviousState`、调用旧 end、新 begin、替换或取消协程，见 `StateMachine.cs:28-75`。Rust 快照恢复时若直接从字段构造而错误触发 begin/end，会与原版的“恢复当前运行态”语义不同；需要区分初始化状态与状态迁移。

## 6. Normal 状态摘要

实现范围：`NormalBegin/End` 见 `Player.cs:3531-3541`，主体 `NormalUpdate` 见 `Player.cs:3566-3843`。

### 6.1 进入、退出与状态优先级

- begin 只把 `maxFall` 重置为 160。
- end 清 wall boost、wall speed retention 和 climb-hop 等待水平速度。
- Normal 每帧首先处理上升平台的 lift boost、抓取/进入 Climb、开始 Dash、蹲伏切换；这些分支发生在水平加速度和重力之前。
- 抓墙条件不只是 `grab && adjacent solid`：要求不 tired、不蹲、未持物，纵速非负，水平速度符号不背离朝向，并经过 bounds、`ClimbBlocker`、2 px 实体探测；还支持向上 1..2 px 修正。见 `Player.cs:3572-3606`。
- `CanDash` 成立时先加 `LiftBoost` 再返回 Dash 状态，见 `Player.cs:3608-3612`。

### 6.2 水平运动

- 地面蹲伏：`Speed.X` 以 `500 * dt` 接近 0。
- 其他情况的基础控制倍率为地面 1、空中 0.65；Cold core 地面再乘 0.3，Low Friction 再乘地面 0.35/空中 0.5。
- 普通目标速度 90；携带 `SlowRun` 为 70；空中持 glider 为约 108 且控制倍率再乘 0.5；Space 再将目标速度乘 0.6。
- 若当前速度绝对值已超过目标且输入同向，用 `400 * multiplier * dt` 回落；否则用 `1000 * multiplier * dt` 接近目标。见 `Player.cs:3674-3715`。

### 6.3 重力、快速下落与 wall slide

- `maxFall` 在普通 160 和按下快速下落 240 之间以 `300 * dt` 接近；Space 同时把两个目标乘 0.6。见 `Player.cs:3716-3741`。
- wall slide 要求空中、纵速非负、wall slide timer 大于 0、面向/输入条件满足、相邻 Solid、无 edge blocker 且可站立。目标下落速度为 `lerp(160, 20, wallSlideTimer / 1.2)`。见 `Player.cs:3742-3771`。
- 当 `abs(Speed.Y) < 40` 且 jump held 或 AutoJump 时，重力倍率为 0.5；glider、Space 继续乘倍率。最终 `Speed.Y = Approach(Speed.Y, targetFall, 900 * multiplier * dt)`。见 `Player.cs:3773-3783`。
- 可变跳窗口有效且 jump held/AutoJump 时，`Speed.Y = min(Speed.Y, varJumpSpeed)`；松键立即清窗口。见 `Player.cs:3784-3794`。

### 6.4 跳跃判定

- jump pressed 时先尝试土狼时间普通跳，再依左右墙探测选择 climb jump、super wall jump 或普通 wall jump，最后才尝试水面跳。见 `Player.cs:3795-3841`。
- 普通 Jump 消耗输入 buffer，清土狼时间和 dash attack，设置 `varJumpTimer=0.2`、`Speed.X += 40*moveX`、`Speed.Y=-105`，再叠加 `LiftBoost` 并保存 `varJumpSpeed`。见 `Player.cs:2436-2450`。
- 墙跳设置水平 130、纵向 -105，并可能设置 0.16 s 强制水平输入；向上 dash 的 wall check 可从 3 px 扩展到 5 px，但会先排除相应朝向尖刺。见 `Player.cs:2521-2582`。
- Super jump 设置 `(260*facing, -105)`；若蹲伏则取消蹲伏并乘 `(1.25, 0.5)`，即常见 hyper 初速 `(325, -52.5)`。见 `Player.cs:2480-2509`。

## 7. Dash 状态摘要

实现范围：`StartDash` 见 `Player.cs:4211-4219`，`DashBegin/End` 见 `Player.cs:4276-4338`，`DashUpdate` 见 `Player.cs:4340-4448`，`DashCoroutine` 见 `Player.cs:4465-4567`。

### 7.1 开始与协程时序

- `CanDash` 要求 dash/crouch-dash pressed、cooldown `<= 0`、`Dashes > 0`，并排除 talk 和特定 booster 条件，见 `Player.cs:1074-1087`。
- `StartDash` 先记录是否为第二条 dash、扣除一条 dash、记录 demo dash 并消费两个 dash buffer，只返回状态 2；方向和速度尚未设置。
- `DashBegin` 记录是否从地面开始，设置 `dashCooldownTimer=0.2`、`dashRefillCooldownTimer=0.1`、`dashAttackTimer=0.3`、wall slide 重置；保存 `beforeDashSpeed` 后把 `Speed` 和 `DashDir` 清零，并按 demo/向下输入切换蹲伏。
- `DashCoroutine` 先 `yield null`。下一次推进时读取 `lastAim`/override，修正极小轴分量，计算 `speed = aim * 240`。若 dash 水平方向与 dash 前同号，且 dash 前水平绝对速度更大，则保留旧水平速度；水中整体乘 0.75。见 `Player.cs:4465-4489`。

### 7.2 Dash 中可执行动作

- 水平 dash 与 JumpThru 重叠且底部差不超过 6 px 时，会竖直贴到平台顶。见 `Player.cs:4384-4392`。
- 水平 dash 且 jump pressed、土狼时间仍有效时触发 SuperJump。见 `Player.cs:4393-4397`。
- 满足向上 dash 角度时允许 SuperWallJump；否则仍允许普通 WallJump/ClimbJump。见 `Player.cs:4399-4441`。
- `DashUpdate` 自身通常返回 2，真正结束状态的是协程。

### 7.3 Dash 结束

- 普通 dash 等待 0.15 s，assist super dash 等待 0.3 s。
- 等待后设置 `AutoJump=true`。若 `DashDir.Y <= 0`，速度改为 `DashDir * 160`；向下 dash 不在这里降到 160。随后若纵速仍为负，再乘 0.75；最后切回 Normal。见 `Player.cs:4545-4567`。
- 因 `dashAttackTimer=0.3` 独立于 0.15 s Dash 状态，角色回到 Normal 后仍可能 `DashAttacking=true`。属性定义见 `Player.cs:1062-1071`。

## 8. Climb 状态摘要

实现范围：`ClimbBegin/End` 见 `Player.cs:3882-3924`，`ClimbUpdate` 见 `Player.cs:3926-4104`，`ClimbHop` 见 `Player.cs:4122-4144`。

- 进入时取消 AutoJump，水平速度清零，纵速乘 0.2，wall slide 重置，设置 0.1 s `climbNoMoveTimer`；最多向面朝方向挪 2 px，使角色贴墙。见 `Player.cs:3882-3908`。
- 更新最先递减 `climbNoMoveTimer`；接地时体力直接回到 110。跳键优先：反向输入为普通墙跳，否则为 climb jump；其次可 Dash；松 grab 则加 LiftBoost 后回 Normal。见 `Player.cs:3926-3955`。
- 墙体消失时，上升中若非 wall booster 会自动 `ClimbHop`，然后回 Normal。见 `Player.cs:3956-3970`。
- 主动上爬目标 -45、下爬目标 80、静止目标 0；边缘 slip 时目标 30；以 `900*dt` 接近。向上遇顶、ledge blocker 或 slip 条件会停止/ClimbHop。见 `Player.cs:3994-4050`。
- `climbNoMoveTimer <= 0` 后，上爬每秒消耗 45.4545441，空中静止每秒消耗 10，下爬不消耗；体力 `<= 0` 时加 LiftBoost 并回 Normal。见 `Player.cs:4056-4103`。
- climb jump 离地时立即扣 27.5，然后复用普通 Jump。若没有水平输入，设置 `wallBoostDir=-Facing` 和 0.2 s 窗口；窗口内若之后输入该方向，会把 `Speed.X` 改为 `130*moveX` 并返还 27.5 体力。见 `Player.cs:2644-2675`、`Player.cs:1560-1569`。
- `ClimbHop` 纵速取 `min(current, -120)`，水平目标 100；若墙是移动 Solid，会先挂接该 Solid，直到离墙才应用保存的水平速度。它还设置 `forceMoveX=0` 0.2 s 和无风 0.3 s。见 `Player.cs:4122-4144`、`Player.cs:1642-1652`、`Player.cs:1683-1695`。

## 9. 碰撞与边角修正

### 9.1 Actor 层基础语义

- 水平和垂直速度位移先累加到子像素余数，再 ties-to-even 舍入成整数像素。
- `MoveHExact`/`MoveVExact` 每次只前进 1 像素并查询碰撞，因此高速移动也不穿过 Solid。
- 水平只碰 Solid；向下垂直移动还碰从外部进入的 JumpThru，向上不碰 JumpThru。见 `Actor.cs:210-289`。
- `OnGround` 同样查询向下的 Solid 或 `CollideCheckOutside<JumpThru>`，见 `Actor.cs:134-153`。

### 9.2 `OnCollideH`

主体见 `Player.cs:3135-3229`：

1. 禁用 curve dash。
2. StarFly 特殊反弹；DreamDash 状态直接忽略常规回调。
3. Dash attacking 且命中对象提供 `OnDashCollide` 时，先处理 Rebound/Bounce/Ignore；RedDash 会把通常结果改为 Ignore。
4. Dash/RedDash 水平撞墙时：若接地且前方位置可蹲，进入蹲伏并不清速度；否则在纯水平速度下搜索上下各 1..4 px 的角修正，并受 `DashCorrectCheck`/LedgeBlocker 限制。
5. 再尝试进入 DreamDash。
6. 普通撞墙把原 `Speed.X` 保存 0.06 s，然后调用实体 `OnCollide`、清 `Speed.X`、dash attack 和 glider boost。RedDash 转 HitSquash。

### 9.3 `OnCollideV`

主体见 `Player.cs:3231-3416`：

- Dash collide 回调优先于普通地面/天花板响应。
- 向下碰撞时，若处于 Dash/RedDash 且 dash 不是从地面开始，会根据接近零的水平速度向左/右搜索最多 4 px，尝试落地边角修正。见 `Player.cs:3282-3310`。
- 斜下 dash 命地时转成水平 dash：`DashDir=(signX,0)`、`Speed.Y=0`、`Speed.X*=1.2` 并蹲伏。见 `Player.cs:3318-3325`。
- 向上碰撞时，普通搜索左右最多 4 px；竖直 dash attacking 搜索最多 5 px。成功时直接把 Position 改为 `(±offset,-1)`。见 `Player.cs:3360-3387`。
- 撞顶时若 `varJumpTimer < 0.15` 才清零；这保留了 jump 后最初约 0.05 s 的 ceiling grace。见 `Player.cs:3389-3392`。
- 无修正时调用对象 `OnCollide`，清 dash attack/glider boost，并将 `Speed.Y=0`。

### 9.4 位移前的额外修正

- 空中上升且与 JumpThru 重叠时，以 `-40*dt` 向上辅助脱离，见 `Player.cs:1787-1790`。
- 水平 dash 在脚下 3 px 内有 Solid/JumpThru 且不受 ledge blocker 阻止时，直接向下挪 3 px，见 `Player.cs:1791-1794`。

因此，碰撞不能只实现 AABB 速度清零；边角搜索次序、DreamBlock/实体回调、JumpThru outside 语义、hurtbox 检查以及子像素余数都属于可观察行为。

## 10. 关键计时器的设置与递减

| 计时器 | 典型设置点 | 递减/消费点 | 语义 |
|---|---|---|---|
| `jumpGraceTimer` | 接地设 0.1：`Player.cs:1581-1589` | 全局 Update；Jump 清零：`2438-2440` | 土狼时间 |
| `varJumpTimer` | Jump 0.2：`2436-2449`；wallbounce 0.25：`2607-2622` | 全局 `1613-1616`；Normal held/release：`3784-3793` | 可变跳 |
| `dashCooldownTimer` | DashBegin 0.2：`4286` | 全局 `1590-1593` | 禁止立即再次开始 dash |
| `dashRefillCooldownTimer` | DashBegin 0.1：`4287` | 全局 `1594-1612` | 延迟接地/水中补 dash |
| `dashAttackTimer` | DashBegin 0.3：`4296` | 全局 `1577-1580`；碰撞/跳跃清零 | Dash 攻击判定，可跨越状态结束 |
| `wallSlideTimer` | 默认/落地/跳跃重置 1.2 | 仅上一帧 `wallSlideDir != 0` 时在 `1555-1559` 递减 | wall slide 下落目标插值 |
| `forceMoveXTimer` | WallJump 0.16：`2565-2569`；ClimbHop 0.2：`4139-4141` | 全局 `1632-1641` | 覆盖实际水平输入 |
| `wallSpeedRetentionTimer` | 水平碰撞 0.06：`3210-3214` | 全局 `1667-1681` | 离墙后恢复撞前水平速度 |
| `wallBoostTimer` | ClimbJump 0.2：`2654-2658` | 全局 `1560-1569` | 延迟输入返还体力并给墙跳水平速度 |
| `climbNoMoveTimer` | ClimbBegin 0.1：`3887-3889` | ClimbUpdate 首句：`3926-3929` | 抓墙初期锁纵向并免耗体力 |
| `noWindTimer` | ClimbHop 0.3：`4140-4143` | 全局 `1653-1656` | 暂停风影响 |

计时器应允许变成略小于 0 的 `f32`，不要一律 clamp 到 0；原码只有部分计时器使用 `Math.Max`。

## 11. BinaryPacker `.bin` 格式

### 11.1 文件布局

写入顺序见 `BinaryPacker.cs:100-117`，读取见 `BinaryPacker.cs:270-285`：

```text
dotnet_string magic       // writer 写 "CELESTE MAP"
dotnet_string package     // 通常为不含扩展名的 map path/name
i16 string_count
dotnet_string strings[string_count]
element root
```

注意：

- `BinaryReader.ReadString`/`BinaryWriter.Write(string)` 使用 UTF-8 字节和 7-bit encoded byte length，不是 `u16 length + UTF-16`。
- 数值原语按 .NET `BinaryReader/BinaryWriter` 的 little-endian 布局。
- `FromBinary` 读取但不校验 magic，见 `BinaryPacker.cs:274`。原版随后由 `MapData` 校验 root package 是否等于预期 mode path，见 `MapData.cs:114-118`。Rust 的公开解析器应同时校验 magic、索引和长度，不能照搬这种信任输入的行为。

### 11.2 Element 布局

递归元素格式由 `WriteElement`/`ReadElement` 定义，见 `BinaryPacker.cs:151-229`、`BinaryPacker.cs:287-343`：

```text
i16 name_string_index
u8 attribute_count
repeat attribute_count:
    i16 key_string_index
    u8 type_tag
    value(type_tag)
i16 child_count
element children[child_count]
```

属性类型：

| tag | 写入值 | 读取后的 C# object | 依据 |
|---:|---|---|---|
| 0 | `bool` 1 byte | `bool` | `BinaryPacker.cs:184-186,303-305` |
| 1 | `u8` | 转成 `int` | `187-189,306-308` |
| 2 | `i16` | 转成 `int` | `190-192,309-311` |
| 3 | `i32` | `int` | `193-195,312-314` |
| 4 | IEEE-754 `f32` | `float` | `196-198,315-317` |
| 5 | `i16` string table index | `string` | `199-201,318-320` |
| 6 | dotnet string | `string` | `217-219,321-323` |
| 7 | `i16 byte_len` + RLE bytes | 解码为 `string` | `208-214,324-328` |

普通 XML 属性写入前按 `bool -> byte -> short -> int -> float -> interned string` 的顺序推断类型，见 `BinaryPacker.cs:231-267`。float parser 允许整数/小数形式但不接受指数形式；无法解析的内容进入字符串表。

### 11.3 innerText 与 RLE

- 叶节点有文本时，writer 伪造键名 `innerText` 的属性。
- 元素名为 `solids` 或 `bg` 时使用 tag 7 RLE；其他叶文本使用 tag 6 直接字符串。见 `BinaryPacker.cs:169-220`。
- RLE 为重复的 `[u8 run_length, u8 character]`，单段最多 255；解码每两字节一组。见 `RunLengthEncoding.cs:8-37`。
- encoder 把 C# `char` 强制转成 byte，因此该 RLE 只安全保真 0..255 字符；官方 tile 文本是 ASCII，Rust 写入器应拒绝或明确截断策略，而非静默损坏 Unicode。

### 11.4 实现陷阱

- string table 索引和 child count 都是有符号 `i16`；attribute count 是 `u8`。写入器没有溢出检查。
- `IgnoreAttributes` 只忽略 `_eid`，见 `BinaryPacker.cs:66`。
- reader 使用静态 `stringLookup`，原版实现本身不是线程安全的，见 `BinaryPacker.cs:72-76,277-282`。Rust 纯函数解析器不应复制该全局状态。
- root 的 `Package` 字段只在完整文件读取后赋值，子元素没有 package，见 `BinaryPacker.cs:282-284`。
- 原版 reader 没有未知 tag、奇数 RLE、负长度、越界 string index 或递归深度保护。面对用户上传地图时必须增加验证。

## 12. MapData / LevelData / EntityData 解释规则

### 12.1 MapData

- 文件路径为 `Content/Maps/<ModeData.Path>.bin`，见 `MapData.cs:43-45`。
- root package 必须匹配 `ModeData.Path`；root 子元素识别 `levels`、`Filler`、`Style`，见 `MapData.cs:107-177`。
- `Style` 可提供背景色，并保存 `Backgrounds`/`Foregrounds` 元素树；物理首阶段无需实例化 Backdrop，但解析器应保留未知树或明确忽略策略。
- Map bounds 是所有 level bounds 和 filler bounds 的并集，再四周扩 64 px，见 `MapData.cs:192-235`。
- TileBounds 使用 8 px tile，宽高向上取整，见 `MapData.cs:47`。

### 12.2 LevelData

- level 基础字段来自元素属性，`name` 会无条件 `Substring(4)`，见 `LevelData.cs:113-138`。这反映原版 level 名含 `lvl_` 前缀；通用解析器应验证长度/前缀。
- 高度 184 被特殊改写为 180，见 `LevelData.cs:132-137`。若目标是复现原版运行时 bounds，必须保留；若目标是无损编辑，则还需另存原始值。
- `entities/player` 不进入 `Entities`，而是生成绝对 spawn：`level.Bounds.position + local player position`，见 `LevelData.cs:218-251`。
- 非 player entity 和 trigger 都经 `CreateEntityData`；entity 的 position、origin、nodes 保持 level-local 坐标，见 `LevelData.cs:321-387`。
- `solids`、`bg`、`fgtiles`、`bgtiles`、`objtiles` 从 `innerText` 读取，见 `LevelData.cs:297-316`。
- `Dummy = Spawns.Count <= 0`，见 `LevelData.cs:318`；Map transition 会拒绝 Dummy level，见 `MapData.cs:92-100`。
- `LevelData.Check` 使用左/上包含、右/下不包含的矩形判定，见 `LevelData.cs:390-397`。

### 12.3 EntityData

- 结构字段：`id/name/level/position/origin/width/height/nodes`；其他属性保存在 `Values`，见 `EntityData.cs:9-27` 和 `LevelData.cs:321-387`。
- `Float` 使用 invariant culture；`Enum` 忽略大小写；`Int/Bool/Char/HexColor` 各有默认值规则，见 `EntityData.cs:58-162`。
- `Values` 仅在遇到非结构属性时创建；因此只包含结构属性的实体可能得到 `Values == null`。`Has` 却没有 null guard（`EntityData.cs:58-61`），Rust 无需复制这个空指针风险，可用空 map 表达。
- `NodesOffset` 只返回 nodes；`NodesWithPosition` 把 entity position 放在数组首项，见 `EntityData.cs:29-56`。不同实体构造器会选择不同语义，不能在通用 parser 层擅自把 position 合并进 nodes。

## 13. 首阶段可实现且可诚实声明的范围

建议第一阶段目标命名为“Normal/Dash/Climb 静态几何垂直切片”，而不是“完整 1:1 Celeste 物理”。其边界如下：

### 13.1 可实现范围

- 固定普通时间步、无 assist、无 cutscene、无 freeze/time-rate 变化。
- 状态 0 Normal、1 Climb、2 Dash，以及这些状态之间的 Jump、WallJump、ClimbJump、SuperJump、SuperWallJump 转换。
- 完整保存并恢复上述状态所需的子像素余数和内部计时器。
- 站立/蹲伏 collider，静态轴对齐 Solid，单向静态 JumpThru。
- Player 中与这些静态几何相关的水平/垂直碰撞、4/5 px corner correction、3 px floor snap、6 px JumpThru nudge、wall speed retention、土狼时间、可变跳、wall slide、climb stamina。
- `.bin` 安全读取：magic、package、string table、全部 0..7 属性类型、递归 Element、RLE、levels、level bounds/spawn、tile strings、通用 entity/trigger 数据。
- 从 `solids` 文本建立 8 px 静态碰撞网格；保留未知实体及属性，不宣称已模拟其行为。
- 用极小手工地图做逐帧 golden tests，并通过真实 1.4.0.0 游戏/Mod 采集的快照对标。

### 13.2 第一阶段明确排除

- Swim、Boost/RedDash、DreamDash、StarFly、Launch 等其余状态。
- 移动 Solid、LiftBoost 的完整来源与平台 carrying/pushing/squish。
- DreamBlock、Spikes、Water、Booster、Wind、WallBooster、SwapBlock、NegaBlock、LedgeBlocker、ClimbBlocker 等实体行为；即使能解析其 EntityData，也不等于实现物理。
- Dash target 的 `OnDashCollide` 多态回调和所有 entity `OnCollide` 副作用。
- Holdable/glider、低摩擦、Cold core、Space、assist、无限 dash/stamina、demo/crouch dash 的全部组合。
- level transition、bounds enforcement、death/hurtbox/entity PlayerCollider、trigger、水下和摄像机逻辑。
- `.bin` 无损 round-trip 编辑器。原版运行时解析会丢失 XML 属性原始文本类型、`lvl_` 前缀语义和 184 高度原值；无损编辑需要独立 raw AST。

## 14. 当前不能声称 1:1 的差距

即使第一阶段单元测试全部通过，以下任一项未完成时都不应使用“完全还原”“逐帧 1:1”或“位级一致”表述：

1. **快照不闭包**：缺 movement remainder、状态协程进度、输入 buffer 年龄、平台 lift 状态或上述隐字段时，纯函数输入不足以唯一决定下一帧。
2. **输入模型过窄**：需求书的 `jump_pressed/jump_held/dash_pressed/grab_held` 没有 crouch dash、aim vector/lastAim、dash buffer、jump buffer、talk 冲突、glider move 等；八方向 dash 也不能仅由 `move_x/move_y` 在同一帧临时推断。
3. **状态缺失**：需求书枚举少了 23..25，且尚未移植 3..25 的绝大多数状态。
4. **协程时序缺失**：Dash 的 `yield null` 和浮点等待计时、状态 begin/end 同步副作用若被平铺，会产生至少一帧差异。
5. **碰撞世界不完整**：Player 回调依赖 Solid/JumpThru 子类、移动平台、DreamBlock、blocker、spikes 和实体回调。只有 tile AABB 不能覆盖原版关卡。
6. **Actor/Platform 语义未移植**：子像素舍入、逐像素碰撞、outside JumpThru、pusher/squish、LiftSpeed 都在 `Player.cs` 之外。
7. **浮点/数学差异未验证**：Rust `f32` 本身不自动保证与 .NET/XNA 的 `Math.Round`、`Math.Sign`、`Math.Pow`、`Vector2.Normalize` 和编译器中间精度完全一致。
8. **时间模型不完整**：原版 `DeltaTime` 受 TimeRate/Freeze/DashAssist 影响；只提供“frames”而没有明确每帧 dt/冻结语义，无法覆盖所有原版执行路径。
9. **地图兼容不等于实体兼容**：能读全部 Element/EntityData，只能声明“容器格式可解析”；只有所有相关 entity factory 和行为完成后，才能声明官方地图物理兼容。
10. **缺真实游戏逐帧证据**：源码审计只能给出实现依据；最终保真声明必须由同版本游戏采集的逐帧位置、速度、状态、计时器和碰撞事件验证。

## 15. 推荐的验收证据

第一阶段至少应保存以下测试证据：

- BinaryPacker：官方 `.bin` 解码后 root/package/字符串表/element tree 与 C# 工具输出一致；恶意长度、未知 tag、越界索引被安全拒绝。
- Actor：正负子像素累积、恰好 `.5` ties-to-even、撞击清余数、向下 JumpThru outside 语义。
- Normal：地面跑、空中控制、土狼跳、短跳/长跳、wall slide 速度曲线、普通墙跳。
- Dash：按键进入当帧/下一帧时序、八方向初速、保留更快同向水平速度、0.15 s 结束、向上结束乘 0.75、斜下落地转水平。
- Corner correction：水平 dash 上下搜索顺序、向下落地左右搜索顺序、普通/竖直 dash 撞顶 4/5 px 搜索。
- Climb：0.1 s no-move、上/静止/下体力曲线、climb jump 消耗与 wall boost 返还、climb hop。
- 每个用例同时对比最终值和逐帧值；容差测试可用于开发诊断，但“位级一致”必须另设逐字段 bit comparison，且只在已证明相同输入、dt、平台世界和数学语义时使用。

---

本审计的合理用途是界定可移植的确定性内核与待补依赖。它不构成当前实现已经达到原版 1:1 的证明。
