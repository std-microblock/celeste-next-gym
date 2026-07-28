import type { GymMap, SimState } from '../../model.ts'
import type { TrainingDocument, TrainingTechnique, TrainingVariant } from '../catalog.ts'
import { hold, press, room, snapshot } from '../helpers.ts'

function hyperVariant(options: {
  id: string
  title: string
  summary: string
  map: GymMap
  initial: SimState
  success?: string[]
  routeValidation?: { observeUntil: number | string; success: string[] }
}): TrainingVariant {
  const success = options.success ?? ['!final.dead', 'final.speed.x >= 320']
  const document: TrainingDocument = {
    version: 1,
    id: `hyper-${options.id}`,
    technique_id: 'hyper',
    variant_id: options.id,
    variant_title: options.title,
    title: 'Hyper',
    summary: options.summary,
    entry: {
      input_id: 'diagonal_dash', hint: '按住右下并冲刺。',
      check: ['!current.dead', 'current.state == state::dash'],
      failure: { title: '需要右下冲', body: '按住右和下，再按 Dash。' }
    },
    fuzz: {
      version: 1,
      inputs: [hold('hold_diagonal', ['right', 'down']), press('diagonal_dash', ['dash'], 0), press('jump', ['jump'], 'jump_frame')],
      variables: [{ name: 'jump_frame', range: { from: 0, to: 60 } }],
      // F0–F3 Jump presses are buffered during freeze and execute on F4's
      // update, whose post-step snapshot is F5.
      observe_until: 'max(jump_frame+1, 5)',
      success,
      objectives: [{ type: 'maximize', expression: 'final.speed.x' }],
      search: { bindings: {}, output: ['best', 'windows', 'coverage'] },
    },
    teaching: {
      steps: [
        { prompt: '按住右下并冲刺。', order_error: { title: '先右下冲', body: '第一步需要右下冲刺。' }, window_error: { title: '冲刺时机不对', body: '从训练开始时输入右下冲。' } },
        { prompt: '接近最佳点时按 Jump。', order_error: { title: '这里需要 Jump', body: '右下冲后的下一关键输入是跳跃。' }, window_error: { title: '跳跃错过窗口', body: '查看时间线中的绿色可行区间。' } },
      ]
    },
    assist: { result_sample_after_input_frames: 0, auto_slowdown: { enabled_by_default: true, radius_frames: 12, minimum_multiplier: .85 } },
  }
  return {
    id: options.id,
    title: options.title,
    summary: options.summary,
    map: options.map,
    initial: options.initial,
    document,
    validationFuzz: options.routeValidation ? { ...document.fuzz, observe_until: options.routeValidation.observeUntil, success: options.routeValidation.success } : document.fuzz,
  }
}


const openGapMap = room('Hyper · 跨越断层', 'hyper-open-gap', {
  width: 640,
  floorSegments: [
    { x: 0, y: 240, width: 176, height: 30 },
    { x: 208, y: 240, width: 432, height: 30 },
    { x: 176, y: 262, width: 32, height: 8 },
  ],
})
const spikeMap = room('Hyper · 越过尖刺', 'hyper-spikes', {
  entities: [{ kind: 'spikes', bounds: { x: 174, y: 237, width: 80, height: 3 }, direction: { x: 0, y: -1 }, name: 'spikesUp' }],
})
const bubbleMap = room('Hyper · 泡泡起手', 'hyper-bubble', {
  spawn: { x: 112, y: 226 },
  entities: [{ kind: 'booster', bounds: { x: 104, y: 216, width: 16, height: 16 }, direction: { x: 0, y: 0 }, name: 'booster' }],
})

export const hyper: TrainingTechnique = {
  id: 'hyper',
  title: 'Hyper',
  summary: '右下冲后快速起跳，把冲刺转成高速地面移动。',
  related: ['wavedash', 'super'],
  variants: [
    hyperVariant({
      id: 'open-gap', title: '跨越断层', summary: '地面中间缺了一大片，用 Hyper 跨到另一侧。', map: openGapMap, initial: snapshot(openGapMap.spawn),
      routeValidation: { observeUntil: 'jump_frame+25', success: ['!final.dead', 'final.pos.x >= 208'] },
    }),
    hyperVariant({
      id: 'charged-hyper', title: '充能Hyper', summary: '充能Hyper', map: openGapMap, initial: snapshot(openGapMap.spawn),
      success: ['!final.dead', 'final.dashes >= 1', 'final.speed.x >= 320'],
      routeValidation: { observeUntil: 'jump_frame+25', success: ['!final.dead', 'final.pos.x >= 208'] },
    }),
    hyperVariant({
      id: 'spike-gap', title: '越过尖刺', summary: '控制起跳窗口，越过地面的尖刺带。', map: spikeMap, initial: snapshot(spikeMap.spawn),
      routeValidation: { observeUntil: 'jump_frame+20', success: ['!final.dead', 'final.pos.x > 174'] },
    }),
    hyperVariant({
      id: 'bubble-exit', title: '泡泡起手', summary: '从泡泡状态离开后建立 Hyper 节奏。', map: bubbleMap,
      initial: snapshot(bubbleMap.spawn, {
        state: 'Boost', on_ground: false, boost_target: { x: 112, y: 226 }, last_booster_target: { x: 112, y: 226 }, booster_reuse_timer: .45, state_timer: .28,
      }),
      routeValidation: { observeUntil: 'jump_frame+30', success: ['!final.dead', 'final.pos.x > 180'] },
    }),
  ],
}
