# Celeste Tech 覆盖 Checklist

> 面向人类阅读、带中英文说明且每个技巧一个文件的版本见 [`tech-handbook/main.typ`](tech-handbook/main.typ)。本 Markdown 继续作为项目指令要求的固定 120 项编号与覆盖基线；实现状态和证据展示由 Typst 手册承载。

> 本文件是项目内固定的技巧覆盖基线。清单来自 Celeste Wiki `Tech` 页面最后修订于 **2026-05-28** 的文本快照，共 **120** 项。后续实现与审计以本文件为准，不需要再次访问在线 Wiki；只有用户明确要求刷新时才更新基线。

来源记录：<https://celeste.ink/wiki/Tech>

## 完成标准

只有以下四类证据全部存在并通过审计，左侧复选框才可以改为 `[x]`：

1. 已逐段对照带注释的 [`Player.cs`](https://raw.githubusercontent.com/NoelFB/Celeste/refs/heads/master/Source/Player/Player.cs) 和必要的实体源码；
2. Rust 中存在对应实现，且没有用隐藏状态或测试专用捷径代替真实机制；
3. 有针对该技巧关键帧、速度或状态不变量的 Rust 回归测试；
4. 有通过标准实体与正常输入触发的真实 Celeste/Everest E2E 场景，并比较位置、速度、状态、朝向、Dash、体力、接地、蹲伏和死亡，数值误差不超过 `0.01`。

仅有通用移动代码、相似场景、名字相同的 E2E，或者人工注入隐藏状态，都不能单独记为完成。“E2E 候选”只是待复核线索，不计入覆盖率。

每项格式：`编号 技巧 — 状态；源码；Rust；单测；E2E`。

## 当前统计

| 结论 | 数量 |
| --- | ---: |
| 已证明（可勾选） | 12 |
| 有 E2E 候选、仍待完整核证 | 2 |
| 尚未开始逐项审计 | 106 |
| 总计 | 120 |

## 明确排除（不进入 120 项分母）

| 功能 | 结论 | 原因 |
| --- | --- | --- |
| `FinalBoss` 实体链 | 放弃支持 | 剧情 Boss 行为，不属于 Tech 技巧覆盖目标；用户明确要求不做。 |
| `PlayerState::Attract` | 放弃支持 | 仅服务于 `FinalBoss` 命中协程；保留枚举解析，但模拟时返回明确的 `UnsupportedState`。 |

## 1. Mechanics（12 项）

- [ ] `1.1` Berry Mechanics — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `1.2` Climbhop — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `1.3` Corner Correction — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [x] `1.4` Coyote Time/Jump — 状态：已证明；源码：`Player.Update` / `NormalUpdate`；Rust：`step` / `normal_update`；单测：`coyote_jump_consumes_source_grace_window_after_leaving_a_ledge`；E2E：`coyote-jump`
- [ ] `1.5` Dash Attack — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `1.6` Directional Spikes — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `1.7` Fastbubbling — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [x] `1.8` Fastfalling (Fastfall) — 状态：已证明；源码：`Player.NormalUpdate`；Rust：`normal_update`；单测：`fastfall_approaches_source_240_terminal_speed`；E2E：`fastfall`
- [x] `1.9` Input Buffering — 状态：已证明；源码：`Player.NormalUpdate` / `Input.Jump.Pressed`；Rust：`step` / `normal_update`；单测：`buffered_jump_fires_on_the_first_grounded_update`；E2E：`buffered-jump`
- [ ] `1.10` Liftboost — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `1.11` Screen Transition — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [x] `1.12` Wind Resistance — 状态：已证明；源码：`Player.WindMove`；Rust：`advance_wind_controller` / `apply_wind_movement`；单测：`grounded_ducking_blocks_horizontal_wind_movement`；E2E：`playground-wind-ground-ducking`

## 2. Dash Tech（17 项）

- [ ] `2.1` Spring Cancel — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [x] `2.2` Superdash (Super) — 状态：已证明；源码：`Player.DashUpdate` / `SuperJump`；Rust：`dash_update` / `super_jump`；单测：`superdash_sets_source_launch_speed_and_spends_dash`；E2E：`super`
- [x] `2.3` Hyperdash (Hyper) — 状态：已证明；源码：`Player.DashCoroutine` / `SuperJump`；Rust：`dash_update` / `super_jump`；单测：`hyperdash_applies_duck_super_multipliers`；E2E：`hyper`
- [x] `2.4` Wavedash — 状态：已证明；源码：`Player.OnCollideV` / `DashUpdate` / `SuperJump`；Rust：`move_exact` / `dash_update` / `super_jump`；单测：`wavedash_landing_converts_down_diagonal_dash_to_hyper`；E2E：`wavedash`
- [x] `2.5` Extended Dashes — 状态：已证明；源码：`Player.Update` / `DashUpdate`；Rust：`step` / `dash_update`；单测：`extended_super_refills_dash_before_late_dash_jump`；E2E：`extended-super`
- [x] `2.6` Reverse Dashes — 状态：已证明；源码：`Player.Update` / `SuperJump`；Rust：`step` / `super_jump`；单测：`reverse_super_uses_jump_frame_facing_not_dash_direction`；E2E：`reverse-super`
- [ ] `2.7` Superwave — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `2.8` Ultradash (Ultra) — 状态：有候选证据，未完成审计；源码：—；Rust：—；单测：—；E2E：`ultra`（候选，待核证）
- [ ] `2.8.1` Chained Ultras — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `2.8.2` Grounded Ultras — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `2.8.2.1` Grounded Ultra Cancel — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `2.8.3` Delayed Ultra — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `2.9` Demodash (Demo) — 状态：有候选证据，未完成审计；源码：—；Rust：—；单测：—；E2E：`demodash`（候选，待核证）
- [x] `2.9.1` Demohyper — 状态：已证明；源码：Everest `CrouchDash` patch + `Player.SuperJump`；Rust：`begin_dash` / `super_jump`；单测：`demohyper_uses_crouched_super_launch_from_horizontal_demo`；E2E：`demohyper`
- [x] `2.9.2` Up Diagonal Demo (Diag Demo) — 状态：已证明；源码：Everest `CrouchDash` patch + `Player.DashCoroutine`；Rust：`begin_dash` / `dash_update`；单测：`upward_diagonal_demo_keeps_crouched_dash_hitbox`；E2E：`up-diagonal-demo`
- [x] `2.10` Wallbounce (wounce, wb) — 状态：已证明；源码：`Player.DashUpdate` / `SuperWallJump`；Rust：`dash_update` / `super_wall_jump`；单测：`wallbounce_sets_super_wall_jump_speed_and_var_window`；E2E：`wallbounce`
- [ ] `2.10.1` Spiked Wallbounce — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—

## 3. Dashless Tech（27 项）

- [ ] `3.1` Bunnyhop (Bhop) — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.2` Cornerkick — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.3` Ceiling Pop (cpop) — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.4` Crouch Jumps — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.5` Neutral Jump (Neutral) — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.6` 5jump — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.7` Cornerboost (cb) — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.7.1` Downward Cornerboosts — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.7.2` 6jump — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.7.3` Double Cornerboost (dcb) — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.7.4` 7jump — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.7.5` 8jump — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.7.6` 9jump — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.7.7` 11jump — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.7.8` Reverse Cornerboost (rcb) — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.7.9` Neutral Reverse Cornerboost (nrcb) — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.7.10` Spiked Cornerboost — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.7.11` Disappearing Block Cornerboost — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.8` Spike Climb — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.8.1` Narrow Spiked Climb — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.9` Spike Clip — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.10` Spike Jumps — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.11` Stamina Cancel — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.12` Wallboost — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.12.1` Cornerboost Wallboost (cobwob) — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.12.2` Wallboost Neutral — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `3.13` Cornerslip — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—

## 4. Entity Tech（47 项）

- [ ] `4.1` Archie — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.2` Bubble Super / Hyper — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.3` Bumper Clip — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.4` Explosion Boost — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.5` Fish / Iceball / Oshiro / Seeker / Snowball Jump — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.6` Cloud Jump / Spiked Cloud Jump — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.6.1` Cloud Hyper/Super — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.6.2` Cloud Hyper Bunnyhop — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.7` Core Hyper/Super — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.8` Delayed Blockboost — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.9` Dream Grab — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.10` Dream Jump — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.10.1` Dream Double-Jump — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.10.2` Dream Hyper — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.10.3` Dream Smuggle — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.10.3.1` Dream Grab Hyper — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.10.3.2` Holdable Dream Hyper — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.10.4` Holdable Grabless Dream Hyper — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.11` Holdable Core Super/Hyper — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.12` Featherboost — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.13` Feather Super — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.14` Heart Ultras — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.15` Jumpthrough Clip — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.15.1` Feather Clip — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.15.2` Feather Hitbox Preservation — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.16` Lava Neutrals — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.17` Moon Boost — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.18` Reform Tech — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.18.1` Reform Kick — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.18.2` Reform Boost (Cassette Boost) — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.18.2.1` Cassoosted Fuper — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.18.3` Core Block Entity Displacement (ced) — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.19` Seeker Bounce — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.20` Theo/Jelly Regrabs — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.21` Holdable Slash — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.22` Neutral Drop — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.22.1` Holdable Stall — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.22.2` Holdable Climb — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.22.3` Holdable Neutral Jump — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.22.4` Holdable Laddering — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.23` Theo/Jelly Ultras — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.24` Holdable Dash Smuggle (From bumpers) — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.25` Throwable Backboost (Backboost) — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.26` Jellyvator / Theovator — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.27` Waterboost — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.28` Koral Clip — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `4.29` Springboost cancel — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—

## 5. Other Tech（17 项）

- [ ] `5.1` Bino Tech — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `5.1.1` Bino Clip — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `5.1.2` Bino Control Storage — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `5.1.3` Bino Interaction Storage — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `5.1.4` Bino Extensions — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `5.2` Bubsdrop — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `5.3` Cassette Raise — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `5.4` Cutscene Warps — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `5.5` Half Stamina Climbing — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `5.6` Kermit Dash — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `5.7` Pause Buffering — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `5.8` Roboboost — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `5.9` Screen Transition Cassette Offset — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `5.10` Spinner Stunning — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `5.11` Spinner Freeze — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `5.12` Subpixel Manipulation — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—
- [ ] `5.13` Undemo (omed) Dashing — 状态：未审计；源码：—；Rust：—；单测：—；E2E：—

## 维护规则

- 每次完成一项审计时，同时填写源码入口、Rust 文件/符号、单测名称与 E2E 场景名称，然后再勾选。
- 若结论为“不支持”，必须记录产品理由，并明确它是否仍计入 120 项覆盖率分母；不能用“不支持”伪装成“已覆盖”。
- 新增或重命名技巧只能在用户明确要求刷新在线来源后进行，并同时更新页面修订日期、分组数量和总数。
- 宣称 100% 覆盖前，必须确认本文件恰好为 `120/120` 个 `[x]`，且不存在空证据字段。
