# 写一个训练技巧

训练系统已经改为地图驱动：一张地图可以通过多个 `GymMap.entities` 中的 `training_trigger` 自定义实体挂载多个教程/Fuzz 模块，教程不再各自拥有地图。

完整制作流程、Trigger 原理、JSON 示例、全部脚本字段、表达式上下文、验证命令和结算指标统一维护在 [教程地图制作手册](./authoring-training-maps.md)，请勿再按旧的“一教程一 Variant”格式新增内容。
