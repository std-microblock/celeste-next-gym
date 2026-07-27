#import "../../template.typ": tech, evidence

#tech(
  id: "3.1",
  title-zh: "Bunnyhop 连跳",
  title-en: "Bunnyhop (Bhop)",
  status: "unimplemented",
  description-zh: [在接地瞬间起跳可减少地面摩擦并保留高速；普通连续兔跳还会重复利用跳跃的水平加速。],
  description-en: [Jumping on the landing frame minimizes ground friction, preserving incoming speed and repeatedly applying jump acceleration.],
  source-evidence: evidence(
    path: [Source/Player/Player.cs],
    symbol: [Player.Update; Player.NormalUpdate; Player.Jump],
    snippet: raw(block: true, lang: "cs", "if (onGround)\n    jumpGraceTimer = JumpGraceTime;\n...\nif (Math.Abs(Speed.X) > max && Math.Sign(Speed.X) == moveX)\n    Speed.X = Calc.Approach(Speed.X, max * moveX, RunReduce * mult * Engine.DeltaTime);\n...\nif (Input.Jump.Pressed && jumpGraceTimer > 0)\n    Jump();\n...\nSpeed.X += JumpHBoost * moveX;"),
    note: [Player.Update 在状态回调前识别接地并恢复 0.1 秒跳跃宽限。NormalUpdate 先执行一帧地面 RunReduce，再消费缓冲跳；Jump 随后追加 40 水平速度，因此接地帧立即起跳能显著减少持续地面减速。],
  ),
  rust-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [step; normal_update],
    note: [模拟器在每帧状态回调前刷新 grounded 与 jump grace，保留 0.1 秒 jump buffer，并按源码顺序执行地面减速后追加 JUMP_H_BOOST。],
  ),
  test-evidence: evidence(
    path: [crates/celeste-physics/src/sim.rs],
    symbol: [bunnyhop_buffers_the_landing_and_reapplies_horizontal_jump_boost],
    note: [以 160 水平速度下落并预输入跳跃；测试证明首次接地状态后的下一次更新立即起跳，仅承受一帧 RunReduce，并相对不跳对照精确多出 40 水平速度。],
  ),
  e2e-evidence: none,
  candidate-e2e: "bunnyhop",
)
